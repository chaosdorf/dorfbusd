use std::{net::SocketAddr, str::FromStr};

use anyhow::Context;
use axum::{
    extract::OriginalUri,
    handler::Handler,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    AddExtensionLayer, Router,
};
use clap::{crate_authors, crate_name, crate_version};
use config::Config;
use http::{Method, StatusCode, Uri};
use tokio::{fs::File, io::AsyncReadExt};
use tokio_modbus::client::rtu;
use tokio_serial::SerialStream;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, instrument, warn};

use crate::{api::api_routes, state::State};

mod api;
mod bus_state;
mod cli;
mod config;
mod model;
mod state;
mod swagger_ui;
#[cfg(test)]
mod tests;

async fn default_404(method: Method, original_uri: OriginalUri) -> impl IntoResponse {
    warn!(
        method = %method,
        uri = %original_uri.0,
        "HTTP request on unknown path"
    );

    (
        StatusCode::NOT_FOUND,
        Html(include_str!("resources/404.html")),
    )
}

#[instrument]
async fn load_config(path: &str) -> anyhow::Result<Config> {
    let mut config_string = String::new();
    File::open(path)
        .await
        .with_context(|| "Error opening the config file")?
        .read_to_string(&mut config_string)
        .await
        .with_context(|| "Error reading the config file")?;
    match toml::from_str::<Config>(&config_string) {
        Ok(config) => {
            info!(device_count = config.devices.len(), "read config");
            Ok(config)
        }
        Err(err) => Err(err.into()),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let params =
        cli::app().map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

    tracing_subscriber::fmt::init();

    info!(include_str!("motd.txt"));

    let config = load_config(&params.config_path)
        .await
        .context("Could not load config")?;

    info!(
        version = crate_version!(),
        authors = crate_authors!(),
        http_port = %params.port,
        serial_path = %params.serial_path,
        serial_boud = %params.serial_boud,
        "starting {}",
        crate_name!()
    );

    let builder = tokio_serial::new(&params.serial_path, params.serial_boud);
    let port = SerialStream::open(&builder).context("Could not open the serial device")?;

    let modbus_ctx = rtu::connect(port).await?;

    let state = State::new(params.clone(), config, modbus_ctx)?;

    {
        let state = state.clone();
        let _join_handle = tokio::spawn(async move {
            if let Err(err) = state.bus_state().check_state_from_device(&state).await {
                error!("{:?}", err);
            }
        });
    }

    let cors = CorsLayer::permissive();

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(state))
        .layer(cors);

    let app = Router::new()
        .route(
            "/",
            get(|| async { Redirect::found(Uri::from_static("/api/swagger-ui/")) }),
        )
        .nest("/api", api_routes())
        .fallback(default_404.into_service())
        .layer(middleware_stack);

    let addr = SocketAddr::from_str(&format!("[::]:{}", params.port))?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
