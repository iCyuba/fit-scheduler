use reqwest::header::{InvalidHeaderValue, ToStrError};
use thiserror::Error;

use crate::parser::{config::ConfigParseError, wsignin::WSignInParseError};

/// A common error type used in the login flow
#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Expected a redirect")]
    NoRedirect,

    #[error("Too many redirects during login flow")]
    TooManyRedirects,

    #[error("Authorization code is missing (???)")]
    MissingCode,

    #[error(transparent)]
    WSignInParseError(#[from] WSignInParseError<reqwest::Error>),

    #[error(transparent)]
    ConfigParseError(#[from] ConfigParseError<reqwest::Error>),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    ToStrError(#[from] ToStrError),

    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}
