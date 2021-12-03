use async_trait::async_trait;
use thiserror::Error;
use tokio_modbus::client::{Context as ModbusContext, Reader};

#[async_trait]
pub trait DorfbusExt {
    /// Read the hardware version of a relais card.
    ///
    /// Use `set_slave` to select a device.
    async fn read_hardware_version(&mut self) -> DorfbusResult<u16>;
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
}

#[derive(Error, Debug)]
pub enum DorfbusError {
    #[error("Got an empyt response from device")]
    ModbusEmptyResponse,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type DorfbusResult<T> = Result<T, DorfbusError>;
