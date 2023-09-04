use std::fmt;

use actix_web_httpauth::{extractors::AuthenticationError, headers::www_authenticate::bearer::Bearer};

/// Echo Error
#[derive(Debug)]
pub enum EchoError {
    AnyhowError(anyhow::Error),
    SqlError(sqlx::Error),
    AuthError(AuthenticationError<Bearer>)
}
// Implement display trait for `EchoError`
impl fmt::Display for EchoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Think of more meaningful display messages
        write!(f, "error")
    }
}
/// Implement `ResponseError` for actix-web Responder
impl actix_web::error::ResponseError for EchoError {}
/// Implement error conversion (`anyhow::Error` -> `EchoError`)
impl From<anyhow::Error> for EchoError {
    fn from(err: anyhow::Error) -> EchoError {
        EchoError::AnyhowError(err)
    }
}
/// Implement error conversion (`sqlx::Error` -> `EchoError`)
impl From<sqlx::Error> for EchoError {
    fn from(err: sqlx::Error) -> EchoError {
        EchoError::SqlError(err)
    }
}
/// Implement error conversion (`AuthError<Bearer>` -> `EchoError`)
impl From<AuthenticationError<Bearer>> for EchoError {
    fn from(err: AuthenticationError<Bearer>) -> EchoError {
        EchoError::AuthError(err)
    }
}
