use axum::{
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Host not found: {0}")]
    HostNotFound(String),
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    #[error("Path not found: {0}")]
    RouterPathNotFound(String),
    #[error("Method not found: {0}")]
    RouterMethodNotAllow(Method),
    #[error("Serde json error: {0}")]
    SerderError(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = match self {
            AppError::HostNotFound(_) => StatusCode::NOT_FOUND,
            AppError::AnyhowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RouterPathNotFound(_) => StatusCode::NOT_FOUND,
            AppError::RouterMethodNotAllow(_) => StatusCode::METHOD_NOT_ALLOWED,
            AppError::SerderError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, self.to_string()).into_response()
    }
}
