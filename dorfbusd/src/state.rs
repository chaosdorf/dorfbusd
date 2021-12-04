use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;
use tokio_modbus::client::Context as ModbusContext;

use crate::{cli::Params, config::Config};

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
