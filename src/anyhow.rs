use crate::{DynHttpError, HttpError, IntoHttpErrorResponse};
use std::{error::Error, fmt::Display, marker::PhantomData};

/// Wrapper around [anyhow::Error] allowing it to be used as a [HttpError]
/// without exposing the details.
///
/// If the `hide-anyhow` feature is enable errors from anyhow will contain a
/// generic error message rather than the [Display] message
#[derive(Debug)]
pub struct AnyhowHttpError(anyhow::Error);

impl Error for AnyhowHttpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl Display for AnyhowHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl HttpError for AnyhowHttpError {
    #[cfg(feature = "log")]
    fn log(&self) {
        // Anyhow errors contain a stacktrace so only the debug variant is used
        log::error!("{:?}", self.0);
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
        DynHttpError {
            inner: Box::new(AnyhowHttpError(value)),
            _marker: PhantomData,
        }
    }
}
