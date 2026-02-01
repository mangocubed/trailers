use axum::http::{HeaderName, StatusCode};

pub const ERROR_FORBIDDEN: (StatusCode, &str) = (StatusCode::FORBIDDEN, "Forbidden");
pub const HEADER_X_API_TOKEN: HeaderName = HeaderName::from_static("x-api-token");
