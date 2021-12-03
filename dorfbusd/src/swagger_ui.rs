use axum::{
    body::{boxed, BoxBody, Bytes, Full},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use http::{header, HeaderValue, Response};
use mime::{Mime, TEXT_CSS, TEXT_JAVASCRIPT};

#[derive(Clone, Debug)]
pub struct WithContentType<T>(Mime, pub T);

impl<T> IntoResponse for WithContentType<T>
where
    T: Into<Full<Bytes>>,
{
    fn into_response(self) -> Response<BoxBody> {
        let mut res = Response::new(boxed(self.1.into()));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(self.0.as_ref()).expect("mime has invalid encoding"),
        );
        res
    }
}

pub fn swagger_routes() -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("resources/swagger-ui.html")) }),
        )
        .route(
            "/swagger-ui-bundle.js",
            get(|| async {
                WithContentType(
                    TEXT_JAVASCRIPT,
                    include_str!("resources/swagger-ui-bundle.js"),
                )
            }),
        )
        .route(
            "/swagger-ui-standalone-preset.js",
            get(|| async {
                WithContentType(
                    TEXT_JAVASCRIPT,
                    include_str!("resources/swagger-ui-standalone-preset.js"),
                )
            }),
        )
        .route(
            "/swagger-ui.css",
            get(|| async { WithContentType(TEXT_CSS, include_str!("resources/swagger-ui.css")) }),
        )
}
