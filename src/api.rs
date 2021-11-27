use actix_web::{
    get,
    web::{self, Json},
    Responder, Scope,
};
use openapiv3::{OpenAPI, Server};

use crate::{cli::Params, swagger_ui::swagger_scope};

#[get("/openapi.json")]
async fn openapi_json(params: web::Data<Params>) -> impl Responder {
    let mut spec: OpenAPI =
        serde_yaml::from_str(include_str!("openapi.yml")).expect("could not parse openapi spec");

    spec.servers.push(Server {
        url: format!("http://localhost:{}/", params.port),
        description: Some("localhost".to_owned()),
        ..Default::default()
    });

    Json(spec)
}

pub fn api_scope() -> Scope {
    web::scope("/api")
        .service(openapi_json)
        .service(swagger_scope())
}
