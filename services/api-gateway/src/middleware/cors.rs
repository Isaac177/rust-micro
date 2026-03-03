use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderName, Method,
};
use tower_http::cors::{Any, CorsLayer};

pub fn build_cors() -> CorsLayer {
    let request_id_header = HeaderName::from_static("x-request-id");

    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE, request_id_header.clone()])
        .expose_headers([request_id_header])
}
