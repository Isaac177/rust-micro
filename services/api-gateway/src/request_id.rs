use axum::{
    extract::Request,
    http::{header::HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const X_REQUEST_ID: &str = "x-request_id";

#[derive(Clone, Debug)]
pub struct RequestId(String);

impl RequestId {
    pub fn generate() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn from_header(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub async fn set_request_id(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(RequestId::from_header)
        .unwrap_or_else(RequestId::generate);

    request.extensions_mut().insert(request_id.clone());

    let mut response = next.run(request).await;
    let header_name = HeaderName::from_static(X_REQUEST_ID);

    if let Ok(header_value) = HeaderValue::from_str(request_id.as_str()) {
        response.headers_mut().insert(header_name, header_value);
    }

    response

}