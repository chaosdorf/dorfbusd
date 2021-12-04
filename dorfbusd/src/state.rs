use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicBool, Arc},
};

use parking_lot::RwLock;
use tokio::sync::Mutex as TokioMutex;
use tokio_modbus::client::Context as ModbusContext;

use crate::{
    cli::Params,
    config::{self, Config},
};

#[derive(Clone)]
pub struct State {
    inner: Arc<StateInner>,
}

impl State {
    pub fn new(params: Params, config: Config, modbus: ModbusContext) -> State {
        State {
            inner: Arc::new(StateInner {
                params,
                config,
                modbus: TokioMutex::new(modbus),
            }),
        }
    }

    pub fn params(&self) -> &Params {
        &self.inner.params
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn modbus(&self) -> &TokioMutex<ModbusContext> {
        &self.inner.modbus
    }
}

struct StateInner {
    params: Params,
    config: Config,
    modbus: TokioMutex<ModbusContext>,
}

#[derive(Debug, Default)]
pub struct BusState {
    devices: BTreeMap<String, Arc<DeviceState>>,
    coils: BTreeMap<String, Arc<CoillState>>,
}

#[derive(Debug, Default)]
pub struct DeviceState {
    pub config: config::Device,
    pub version: RwLock<Option<u16>>,
    pub seen: AtomicBool,
}

#[derive(Debug, Default)]
pub struct CoillState {
    pub config: config::Coil,
    pub device: Arc<DeviceState>,
    pub status: RwLock<CoilValue>,
}

#[derive(Debug, Copy, Clone)]
pub enum CoilValue {
    On,
    Off,
    Unknown,
}

impl Default for CoilValue {
    fn default() -> Self {
        CoilValue::Unknown
    }
}
