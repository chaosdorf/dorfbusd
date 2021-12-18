use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;
use tokio_modbus::client::Context as ModbusContext;
use tracing::info;

use crate::{
    bus_state::{BusState, CoilState, CoilUpdate},
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

    pub async fn get_tag(&self, name: &str) -> StateResult<Vec<CoilUpdate>> {
        let bus_state = self.bus_state();

        let coil_updates = bus_state
            .tags
            .get(name)
            .ok_or_else(|| StateError::TagNotFound(name.to_string()))?
            .iter()
            .map(|coil_state| CoilState::as_update(coil_state))
            .collect();

        Ok(coil_updates)
    }

    pub async fn set_tag(&self, name: &str, enabled: bool) -> StateResult<Vec<CoilUpdate>> {
        info!("locking modbus device...");
        let mut modbus = self.modbus().lock().await;

        let bus_state = self.bus_state();

        let coils = bus_state
            .tags
            .get(name)
            .ok_or_else(|| StateError::TagNotFound(name.to_string()))?;

        let mut results = Vec::new();
        for coil_state in coils {
            results.push(coil_state.set_coil(&mut modbus, enabled).await);
        }

        let final_result: StateResult<Vec<_>> = results.into_iter().collect();
        Ok(final_result?)
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
    #[error("tag {0:?} not found")]
    TagNotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("got timeout on modbus")]
    Timeout,
}

pub type StateResult<T> = Result<T, StateError>;
