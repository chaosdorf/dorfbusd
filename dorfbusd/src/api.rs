use std::{sync::Arc, time::Duration};

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use dorfbusext::DorfbusExt;
use http::StatusCode;
use openapiv3::{OpenAPI, Server};
use serde_json::json;
use thiserror::Error;
use tokio::{
    sync::Mutex,
    time::{error::Elapsed, timeout},
};
use tokio_modbus::{
    client::{Context, Writer},
    prelude::{Slave, SlaveContext},
};
use tracing::{info, instrument};

use crate::{cli::Params, config::Config, swagger_ui::swagger_routes};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Modbus timed out")]
    ModbusTimeout,
}

impl From<Elapsed> for ApiError {
    fn from(_: Elapsed) -> ApiError {
        ApiError::ModbusTimeout
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> http::Response<axum::body::BoxBody> {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "msg": self.to_string(),
            })),
        )
            .into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

async fn openapi_json(Extension(params): Extension<Arc<Params>>) -> impl IntoResponse {
    let mut spec: OpenAPI =
        serde_yaml::from_str(include_str!("openapi.yml")).expect("could not parse openapi spec");

    spec.servers.push(Server {
        url: format!("http://localhost:{}/", params.port),
        description: Some("localhost".to_owned()),
        ..Default::default()
    });

    (StatusCode::OK, Json(spec))
}

#[instrument(skip_all)]
async fn config(Extension(config): Extension<Arc<Config>>) -> impl IntoResponse {
    Json(config.as_ref().clone())
}

#[instrument(skip(modbus_mutex))]
async fn device_hardware_id(
    Path(device_id): Path<u8>,
    Extension(modbus_mutex): Extension<Arc<Mutex<Context>>>,
) -> ApiResult<impl IntoResponse> {
    info!("locking modbus device...");
    let mut modbus = modbus_mutex.lock().await;

    modbus.set_slave(Slave(device_id));
    let hardware_version_res =
        timeout(Duration::from_secs(1), modbus.read_hardware_version()).await?;
    let hardware_version = hardware_version_res.unwrap();
    Ok(Json(json!({ "hardware-version": hardware_version })))
}

#[instrument(skip(modbus_mutex))]
async fn set_coil(
    Path((device_id, coil, status)): Path<(u8, u16, bool)>,
    Extension(modbus_mutex): Extension<Arc<Mutex<Context>>>,
) -> ApiResult<impl IntoResponse> {
    info!("locking modbus device...");
    let mut modbus = modbus_mutex.lock().await;

    modbus.set_slave(Slave(device_id));
    let coil_status_res = timeout(
        Duration::from_secs(1),
        modbus.write_single_coil(coil, status),
    )
    .await?;
    let _coil_status = coil_status_res.unwrap();
    Ok(Json(json!({})))
}

fn api_v1_routes() -> Router {
    Router::new()
        .route("/config", get(config))
        .route(
            "/device-hardware-version/:device-id",
            get(device_hardware_id),
        )
        .route("/set-coil/:device-id/:coil/:value", get(set_coil))
}

pub fn api_routes() -> Router {
    Router::new()
        .nest("/v1", api_v1_routes())
        .route("/openapi.json", get(openapi_json))
        .nest("/swagger-ui", swagger_routes())
}
