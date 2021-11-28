use std::sync::Arc;

use axum::{extract::Extension, response::IntoResponse, routing::get, Json, Router};
use http::StatusCode;
use openapiv3::{OpenAPI, Server};
use tracing::instrument;

use crate::{cli::Params, config::Config, swagger_ui::swagger_routes};

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
async fn config(Extension(config): Extension<Config>) -> impl IntoResponse {
    Json(config)
}

fn api_v1_routes() -> Router {
    Router::new().route("/config", get(config))
}

pub fn api_routes() -> Router {
    Router::new()
        .nest("/v1", api_v1_routes())
        .route("/openapi.json", get(openapi_json))
        .nest("/swagger-ui", swagger_routes())
}
