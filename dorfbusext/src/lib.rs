use async_trait::async_trait;
use thiserror::Error;
use tokio_modbus::client::{Context as ModbusContext, Reader, Writer};

#[async_trait]
pub trait DorfbusExt {
    /// Read the hardware version of a relais card.
    ///
    /// Use `set_slave` to select a device.
    async fn read_hardware_version(&mut self) -> DorfbusResult<u16>;

    /// Set the device address of a relais card.
    ///
    /// This will send a broadcast command.
    /// Use this only if a single device is connected to the bus.
    async fn set_device_address(&mut self, addr: u8) -> DorfbusResult<()>;
}

#[async_trait]
impl DorfbusExt for ModbusContext {
    async fn read_hardware_version(&mut self) -> DorfbusResult<u16> {
        let hardware_version = self
            .read_holding_registers(0x20, 1)
            .await?
            .into_iter()
            .next()
            .ok_or(DorfbusError::ModbusEmptyResponse)?;
        Ok(hardware_version)
    }

    async fn set_device_address(&mut self, addr: u8) -> DorfbusResult<()> {
        match self.write_single_register(0x4000, addr as u16).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::InvalidData => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Error, Debug)]
pub enum DorfbusError {
    #[error("Got an empyt response from device")]
    ModbusEmptyResponse,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type DorfbusResult<T> = Result<T, DorfbusError>;
