use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(crate) enum ApiError {
    InternalServerError(anyhow::Error),
}

impl From<anyhow::Error> for ApiError {
    fn from(inner: anyhow::Error) -> Self {
        ApiError::InternalServerError(inner)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::InternalServerError(error) => {
                tracing::error!("stacktrace: {}", error.backtrace());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong!".to_string(),
                )
            }
        };

        let body = json!({
            "error": error_message
        });

        tracing::error!("Error: {status:?} with message {error_message:?}");

        (status, Json(body)).into_response()
    }
}
