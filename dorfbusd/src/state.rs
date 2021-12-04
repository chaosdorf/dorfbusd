use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;
use tokio_modbus::client::Context as ModbusContext;

use crate::{bus_state::BusState, cli::Params, config::Config};

#[derive(Clone)]
pub struct State {
    inner: Arc<StateInner>,
}

impl State {
    pub fn new(params: Params, config: Config, modbus: ModbusContext) -> anyhow::Result<State> {
        let bus_state = Arc::new(BusState::try_from(&config)?);

        Ok(State {
            inner: Arc::new(StateInner {
                params,
                config,
                modbus: TokioMutex::new(modbus),
                bus_state,
            }),
        })
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

    pub fn bus_state(&self) -> Arc<BusState> {
        self.inner.bus_state.clone()
    }
}

struct StateInner {
    params: Params,
    config: Config,
    modbus: TokioMutex<ModbusContext>,
    bus_state: Arc<BusState>,
}
