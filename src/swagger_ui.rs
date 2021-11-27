use actix_web::{get, web, HttpResponse, Responder, Scope};

#[get("/")]
async fn swagger_ui() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("resources/swagger-ui.html"))
}

#[get("/swagger-ui-bundle.js")]
async fn swagger_ui_bundle() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(include_str!("resources/swagger-ui-bundle.js"))
}

#[get("/swagger-ui-standalone-preset.js")]
async fn swagger_ui_standalone_preset() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(include_str!("resources/swagger-ui-standalone-preset.js"))
}

#[get("/swagger-ui.css")]
async fn swagger_ui_css() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css")
        .body(include_str!("resources/swagger-ui.css"))
}

pub fn swagger_scope() -> Scope {
    web::scope("/swagger-ui")
        .service(swagger_ui)
        .service(swagger_ui_bundle)
        .service(swagger_ui_standalone_preset)
        .service(swagger_ui_css)
}
