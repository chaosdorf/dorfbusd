use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;
use tokio_modbus::client::Context as ModbusContext;
use tracing::info;

use crate::{
    bus_state::{BusState, CoilUpdate},
    cli::Params,
    config::Config,
};

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

    pub async fn get_coil(&self, name: &str) -> StateResult<CoilUpdate> {
        let bus_state = self.bus_state();
        let coil_state = bus_state
            .coils
            .get(name)
            .ok_or_else(|| StateError::CoilNotFound(name.to_string()))?;
        let coil_update = coil_state.get_coil().await?;
        Ok(coil_update)
    }

    pub async fn set_coil(&self, name: &str, enabled: bool) -> StateResult<CoilUpdate> {
        info!("locking modbus device...");
        let mut modbus = self.modbus().lock().await;

        let bus_state = self.bus_state();
        let coil_state = bus_state
            .coils
            .get(name)
            .ok_or_else(|| StateError::CoilNotFound(name.to_string()))?;
        let coil_update = coil_state.set_coil(&mut modbus, enabled).await?;

        Ok(coil_update)
    }
}

struct StateInner {
    params: Params,
    config: Config,
    modbus: TokioMutex<ModbusContext>,
    bus_state: Arc<BusState>,
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("coil {0:?} not found")]
    CoilNotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("got timeout on modbus")]
    Timeout,
}

pub type StateResult<T> = Result<T, StateError>;
