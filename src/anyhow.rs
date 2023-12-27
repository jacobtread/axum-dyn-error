//! Wrappers and extension traits for anyhow support

use crate::{DynHttpError, HttpError, IntoHttpErrorResponse};
use http::StatusCode;
use std::{error::Error, fmt::Display};

/// Wrapper around [anyhow::Error] allowing it to be used as a [HttpError],
/// contains a [StatusCode] that will be used for the response.
///
/// If the `hide-anyhow` feature is enable errors from anyhow will contain a
/// generic error message rather than the [Display] message
#[derive(Debug)]
pub struct AnyhowHttpError {
    /// The anyhow error
    error: anyhow::Error,
    /// The response status code
    status: StatusCode,
}

impl Error for AnyhowHttpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl Display for AnyhowHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl HttpError for AnyhowHttpError {
    #[cfg(feature = "log")]
    fn log(&self) {
        // Anyhow errors contain a stacktrace so only the debug variant is used
        log::error!("{:?}", self.error);
    }

    fn status(&self) -> StatusCode {
        self.status
    }

    #[cfg(feature = "hide-anyhow")]
    fn reason(&self) -> String {
        // Anyhow errors use a generic message
        "Server error".to_string()
    }
}

/// Allow conversion from anyhow errors into [DynHttpError] by wrapping
/// them with [AnyhowHttpError]
impl<I> From<anyhow::Error> for DynHttpError<I>
where
    I: IntoHttpErrorResponse,
{
    fn from(value: anyhow::Error) -> Self {
        value
            // Give the error a default status
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            // Convert into the dyn error type
            .into()
    }
}

/// Extension for adding a [StatusCode] to an anyhow error
pub trait AnyhowStatusExt {
    /// Add an additional status code to the anyhow error response
    fn status(self, status: StatusCode) -> AnyhowHttpError;
}

impl AnyhowStatusExt for anyhow::Error {
    fn status(self, status: StatusCode) -> AnyhowHttpError {
        AnyhowHttpError {
            error: self,
            status,
        }
    }
}
