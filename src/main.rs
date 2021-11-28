use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use axum::{
    extract::OriginalUri,
    handler::Handler,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    AddExtensionLayer, Router,
};
use clap::{crate_authors, crate_name, crate_version};
use http::{Method, StatusCode, Uri};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

use crate::api::api_routes;

mod api;
mod cli;
mod config;
mod model;
mod swagger_ui;

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let params =
        cli::app().map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

    tracing_subscriber::fmt::init();

    info!(include_str!("motd.txt"));

    info!(
        version = crate_version!(),
        authors = crate_authors!(),
        http_port = %params.port,
        serial_path = %params.serial_path,
        serial_boud = %params.serial_boud,
        "starting {}",
        crate_name!()
    );

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_origin(tower_http::cors::any())
        .max_age(Duration::from_secs(3600));

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(Arc::new(params.clone())))
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
