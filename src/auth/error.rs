use actix_web::{http::StatusCode, HttpResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Not authenticated")]
    NotAuthenticated,
}

impl actix_web::error::ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse<actix_web::dev::Body> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::NotAuthenticated => StatusCode::UNAUTHORIZED,
            // Error::AuthorizationError => StatusCode::FORBIDDEN,
            // _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
