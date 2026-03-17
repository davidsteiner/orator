pub use axum;
pub use bytes;
pub use http;
pub use tower;

#[cfg(feature = "cookies")]
pub use axum_extra;

#[derive(Debug)]
pub struct ParamRejection {
    message: String,
}

impl ParamRejection {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParamRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl axum::response::IntoResponse for ParamRejection {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::BAD_REQUEST, self.message).into_response()
    }
}
