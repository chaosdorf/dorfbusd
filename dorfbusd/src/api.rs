use std::time::Duration;

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use dorfbusext::DorfbusExt;
use http::StatusCode;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::time::{error::Elapsed, timeout};
use tokio_modbus::prelude::{Slave, SlaveContext};
use tracing::{info, instrument};

use crate::{
    state::{State, StateError, StateResult},
    swagger_ui::swagger_routes,
};

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
            Json(ApiErrorResponse {
                short: "todo".into(),
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

impl IntoResponse for StateError {
    fn into_response(self) -> http::Response<axum::body::BoxBody> {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiErrorResponse {
                short: "todo".into(),
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}
/// The response object in case of an error
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// A stable textual identifier of the error
    #[schemars(example = "example_error_short")]
    pub short: String,
    /// A human readable error message
    #[schemars(example = "example_error_message")]
    pub message: String,
}

fn example_error_short() -> &'static str {
    "on_fire"
}

fn example_error_message() -> &'static str {
    "The device is on fire"
}

pub type ApiResult<T> = Result<T, ApiError>;

async fn openapi_json(Extension(state): Extension<State>) -> impl IntoResponse {
    let mut spec: openapiv3::OpenAPI =
        serde_yaml::from_str(include_str!("openapi.yml")).expect("could not parse openapi spec");

    spec.servers.push(openapiv3::Server {
        url: format!("http://localhost:{}/", state.params().port),
        description: Some("localhost".to_owned()),
        ..Default::default()
    });

    (StatusCode::OK, Json(spec))
}

#[instrument(skip_all)]
async fn config(Extension(state): Extension<State>) -> impl IntoResponse {
    Json(state.config().clone())
}

#[instrument(skip_all)]
async fn state(Extension(state): Extension<State>) -> impl IntoResponse {
    Json(state.bus_state())
}

#[instrument(skip(state))]
async fn device_hardware_id(
    Path(device_id): Path<u8>,
    Extension(state): Extension<State>,
) -> ApiResult<impl IntoResponse> {
    info!("locking modbus device...");
    let mut modbus = state.modbus().lock().await;

    modbus.set_slave(Slave(device_id));
    let hardware_version_res =
        timeout(Duration::from_secs(1), modbus.read_hardware_version()).await?;
    let hardware_version = hardware_version_res.unwrap();
    Ok(Json(json!({ "hardware-version": hardware_version })))
}

#[instrument(skip(state))]
async fn get_coil(
    Path(name): Path<String>,
    Extension(state): Extension<State>,
) -> StateResult<impl IntoResponse> {
    let coil_update = state.get_coil(&name).await?;

    Ok(Json(coil_update))
}

#[instrument(skip(state))]
async fn set_coil(
    Json(enabled): Json<bool>,
    Path(name): Path<String>,
    Extension(state): Extension<State>,
) -> StateResult<impl IntoResponse> {
    let coil_update = state.set_coil(&name, enabled).await?;

    Ok(Json(coil_update))
}

#[instrument(skip(state))]
async fn get_tag(
    Path(name): Path<String>,
    Extension(state): Extension<State>,
) -> StateResult<impl IntoResponse> {
    let coil_updates = state.get_tag(&name).await?;

    Ok(Json(coil_updates))
}

#[instrument(skip(state))]
async fn set_tag(
    Json(enabled): Json<bool>,
    Path(name): Path<String>,
    Extension(state): Extension<State>,
) -> StateResult<impl IntoResponse> {
    let coil_updates = state.set_tag(&name, enabled).await?;

    Ok(Json(coil_updates))
}

fn api_v1_routes() -> Router {
    Router::new()
        .route("/config", get(config))
        .route("/state", get(state))
        .route(
            "/device-hardware-version/:device-id",
            get(device_hardware_id),
        )
        .route("/coil/:name", get(get_coil).post(set_coil))
        .route("/tag/:name", get(get_tag).post(set_tag))
}

pub fn api_routes() -> Router {
    Router::new()
        .nest("/v1", api_v1_routes())
        .route("/openapi.json", get(openapi_json))
        .nest("/swagger-ui", swagger_routes())
}
