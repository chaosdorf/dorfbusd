use crate::{
    config::{self, Config},
    state::{State, StateError, StateResult},
};
use dorfbusext::DorfbusExt;
pub use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{self, AtomicBool},
        Arc, RwLock,
    },
    time::Duration,
};
use tokio::{sync::oneshot, time::timeout};
use tokio_modbus::{
    client::Context as ModbusContext,
    client::Writer,
    prelude::{Slave, SlaveContext},
};
use tracing::{info, warn};

#[derive(Serialize, Debug, Default, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct BusState {
    pub devices: BTreeMap<String, Arc<DeviceState>>,
    pub coils: BTreeMap<String, Arc<CoilState>>,
    pub tags: BTreeMap<String, Vec<Arc<CoilState>>>,
}

#[derive(Serialize, Debug, Default, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DeviceState {
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub config: config::DeviceConfig,
    #[serde(default)]
    #[schemars(example = "example_106")]
    pub version: RwLock<Option<u16>>,
    #[serde(default)]
    pub seen: AtomicBool,
}

fn example_106() -> RwLock<Option<u16>> {
    Some(106).into()
}

impl DeviceState {
    /// Reset the state of a device
    pub fn reset(&self) {
        *self.version.write().unwrap() = None;
        self.seen.store(false, atomic::Ordering::Relaxed);
    }

    pub async fn check_state_from_device(
        &self,
        modbus_context: &mut ModbusContext,
    ) -> anyhow::Result<()> {
        modbus_context.set_slave(Slave(self.config.modbus_address));
        if let Ok(hardware_version_res) = timeout(
            Duration::from_secs(1),
            modbus_context.read_hardware_version(),
        )
        .await
        {
            let hardware_version = hardware_version_res?;
            *self.version.write().unwrap() = Some(hardware_version);
            self.seen.store(true, atomic::Ordering::Relaxed);
        } else {
            warn!(
                self.config.modbus_address,
                "Could not read hardware version of device"
            );
        }

        Ok(())
    }
}

#[derive(Serialize, Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CoilState {
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub config: config::CoilConfig,
    #[serde(skip)]
    pub device: Arc<DeviceState>,
    pub status: Arc<RwLock<CoilValue>>,
}

impl CoilState {
    /// Reset the state of the coil
    pub fn reset(&self) {
        *self.status.write().unwrap() = CoilValue::Unknown;
    }

    pub fn as_update(&self) -> CoilUpdate {
        CoilUpdate {
            name: self.name.clone(),
            device: self.device.name.clone(),
            device_id: self.device.config.modbus_address,
            coil_id: self.config.address,
            status: *self.status.read().unwrap(),
        }
    }

    /// Set and write the state of a coil
    pub async fn set_coil(
        &self,
        modbus_context: Arc<tokio::sync::Mutex<ModbusContext>>,
        value: bool,
    ) -> StateResult<CoilUpdate> {
        let (tx, rx) = oneshot::channel();
        let cloned = self.clone();

        tokio::spawn(async move {
            let mut modbus_context = modbus_context.lock().await;
            modbus_context.set_slave(Slave(cloned.device.config.modbus_address));

            info!(value, name = ?cloned.name, "set coil");

            let _tx_result = tx.send(
                match timeout(
                    Duration::from_secs(1),
                    modbus_context.write_single_coil(cloned.config.address, value),
                )
                .await
                {
                    Ok(Ok(())) => {
                        *cloned.status.write().unwrap() = CoilValue::from(value);
                        Ok(cloned.as_update())
                    }
                    Ok(Err(err)) => {
                        *cloned.status.write().unwrap() = CoilValue::Unknown;
                        Err(err.into())
                    }
                    Err(_) => {
                        *cloned.status.write().unwrap() = CoilValue::Unknown;
                        Err(StateError::Timeout)
                    }
                },
            );
        });

        rx.await?
    }

    /// Get the state of a coil
    pub async fn get_coil(&self) -> StateResult<CoilUpdate> {
        Ok(self.as_update())
    }
}

/// Response to a single coil update
#[derive(Serialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CoilUpdate {
    /// Name of the coil
    pub name: String,
    /// Name of the relais card
    pub device: String,
    /// Modbus id of the relais card
    pub device_id: u8,
    /// Id of the coil on the relais card
    pub coil_id: u16,
    status: CoilValue,
}

#[derive(Serialize, Debug, Copy, Clone, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CoilValue {
    On,
    Off,
    Unknown,
}

impl From<bool> for CoilValue {
    fn from(v: bool) -> Self {
        if v {
            CoilValue::On
        } else {
            CoilValue::Off
        }
    }
}

impl Default for CoilValue {
    fn default() -> Self {
        CoilValue::Unknown
    }
}

impl TryFrom<&Config> for BusState {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let devices: BTreeMap<_, _> = config
            .devices
            .iter()
            .map(|(name, device)| {
                (
                    name.to_owned(),
                    Arc::new(DeviceState {
                        name: name.clone(),
                        config: device.clone(),
                        version: RwLock::new(None),
                        seen: AtomicBool::from(false),
                    }),
                )
            })
            .collect();

        let coils_res: anyhow::Result<BTreeMap<_, _>> = config
            .coils
            .iter()
            .map(|(name, coil)| {
                let device = devices
                    .get(&coil.device)
                    .ok_or_else(|| {
                        anyhow::Error::msg(format!(
                            "coil {} is member of device {} which does not exist",
                            name, coil.device,
                        ))
                    })?
                    .clone();
                Ok((
                    name.to_owned(),
                    Arc::new(CoilState {
                        name: name.clone(),
                        config: coil.to_owned(),
                        device,
                        status: Default::default(),
                    }),
                ))
            })
            .collect();

        let coils = coils_res?;

        let mut tags = BTreeMap::new();
        for coil in coils.values() {
            for tag in &coil.config.tags {
                let tag_vec: &mut Vec<_> = tags.entry(tag.clone()).or_default();
                tag_vec.push(coil.clone());
            }
        }

        Ok(BusState {
            devices,
            coils,
            tags,
        })
    }
}

impl BusState {
    /// Reset the state of all devices and coils
    ///
    /// Call this after a powerloss on modbus.
    pub fn reset(&self) {
        self.devices.values().for_each(|state| state.reset());
        self.coils.values().for_each(|state| state.reset());
    }

    pub async fn check_state_from_device(&self, state: &State) -> anyhow::Result<()> {
        let mut modbus_context = state.modbus().lock().await;

        for (name, device) in self.devices.iter() {
            info!(%name, device.config.modbus_address, "read hardware version of device");
            device.check_state_from_device(&mut modbus_context).await?;
        }

        Ok(())
    }
}
