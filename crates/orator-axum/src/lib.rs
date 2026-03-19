pub use axum;
pub use bytes;
pub use chrono;
pub use http;
pub use serde_json;
pub use tower;

pub use axum_extra;

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamLocation {
    Path,
    Query,
    Header,
    Cookie,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParamRejection {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    pub location: ParamLocation,
}

impl ParamRejection {
    pub fn missing(location: ParamLocation, param: impl Into<String>) -> Self {
        let param = param.into();
        Self {
            message: format!("missing required {}: {param}", location.as_str()),
            param: Some(param),
            location,
        }
    }

    pub fn invalid(
        location: ParamLocation,
        param: impl Into<String>,
        detail: impl std::fmt::Display,
    ) -> Self {
        let param = param.into();
        Self {
            message: format!("invalid {} value for {param}: {detail}", location.as_str()),
            param: Some(param),
            location,
        }
    }

    pub fn non_ascii(location: ParamLocation, param: impl Into<String>) -> Self {
        let param = param.into();
        Self {
            message: format!("non-ASCII {} value: {param}", location.as_str()),
            param: Some(param),
            location,
        }
    }
}

impl ParamLocation {
    fn as_str(&self) -> &'static str {
        match self {
            ParamLocation::Path => "path",
            ParamLocation::Query => "query",
            ParamLocation::Header => "header",
            ParamLocation::Cookie => "cookie",
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
        (http::StatusCode::BAD_REQUEST, axum::Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http::StatusCode;
    use http_body_util::BodyExt;

    #[test]
    fn missing_constructor() {
        let r = ParamRejection::missing(ParamLocation::Header, "X-Request-ID");
        assert_eq!(r.message, "missing required header: X-Request-ID");
        assert_eq!(r.param.as_deref(), Some("X-Request-ID"));
        assert!(matches!(r.location, ParamLocation::Header));
    }

    #[test]
    fn invalid_constructor() {
        let r = ParamRejection::invalid(ParamLocation::Query, "page", "not a number");
        assert_eq!(r.message, "invalid query value for page: not a number");
        assert_eq!(r.param.as_deref(), Some("page"));
        assert!(matches!(r.location, ParamLocation::Query));
    }

    #[test]
    fn non_ascii_constructor() {
        let r = ParamRejection::non_ascii(ParamLocation::Header, "Authorization");
        assert_eq!(r.message, "non-ASCII header value: Authorization");
        assert_eq!(r.param.as_deref(), Some("Authorization"));
    }

    #[test]
    fn display_shows_message() {
        let r = ParamRejection::missing(ParamLocation::Cookie, "session");
        assert_eq!(r.to_string(), "missing required cookie: session");
    }

    #[test]
    fn json_serialization_shape() {
        let r = ParamRejection::missing(ParamLocation::Header, "X-Api-Key");
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["message"], "missing required header: X-Api-Key");
        assert_eq!(json["param"], "X-Api-Key");
        assert_eq!(json["location"], "header");
    }

    #[test]
    fn json_serialization_omits_none_param() {
        let r = ParamRejection {
            message: "something went wrong".into(),
            param: None,
            location: ParamLocation::Path,
        };
        let json = serde_json::to_value(&r).unwrap();
        assert!(json.get("param").is_none());
    }

    #[tokio::test]
    async fn into_response_returns_400_json() {
        let r = ParamRejection::missing(ParamLocation::Header, "X-Request-ID");
        let response = r.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json",
        );

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["message"], "missing required header: X-Request-ID");
        assert_eq!(json["param"], "X-Request-ID");
        assert_eq!(json["location"], "header");
    }
}
