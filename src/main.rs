use actix_cors::Cors;
use actix_web::{
    http::header::CONTENT_TYPE,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use clap::{crate_authors, crate_name, crate_version};
use tracing::{info, warn};

use crate::api::api_scope;

mod api;
mod cli;
mod model;
mod swagger_ui;

async fn default_404(request: HttpRequest) -> impl Responder {
    warn!(
        method = %request.method(),
        path = %request.path(),
        query = %request.query_string(),
        "HTTP request on unknown path"
    );

    HttpResponse::NotFound()
        .content_type("text/html")
        .body(include_str!("resources/404.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let params =
        cli::app().map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;

    tracing_subscriber::fmt::init();

    let params_data = web::Data::new(params.clone());

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

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .allowed_header(CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(params_data.clone())
            .service(api_scope())
            .default_service(web::route().to(default_404))
    })
    .bind(format!("[::]:{}", params.port))?
    .workers(1)
    .run()
    .await
}
