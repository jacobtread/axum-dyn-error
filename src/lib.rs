#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::error::Error;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use axum_core::response::{IntoResponse, Response};

// Re-export of status code for ease of use
pub use http::StatusCode;

#[cfg(feature = "anyhow")]
pub mod anyhow;

#[cfg(feature = "anyhow")]
pub use anyhow::*;

/// Alias for [Result] that has a [DynHttpError] as the error type
pub type HttpResult<T, I = TextErrorResponse> = Result<T, DynHttpError<I>>;

/// Structure that stores dynamic error responses, see documentation
/// home page for usage
pub struct DynHttpError<I: IntoHttpErrorResponse = TextErrorResponse> {
    /// The dynamically typed http error that created this error
    inner: Box<dyn HttpError>,
    /// Marker for storing the [IntoHttpErrorResponse] type
    _marker: PhantomData<I>,
}

impl Debug for DynHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(self.inner.type_name())
            .field(&self.inner)
            .finish()
    }
}

impl Display for DynHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error for DynHttpError {}

impl<I: IntoHttpErrorResponse> IntoResponse for DynHttpError<I> {
    fn into_response(self) -> Response {
        let error = self.inner;

        // Log the error if logging is enabled
        #[cfg(feature = "log")]
        {
            error.log();
        }

        // Create the HTTP response
        I::into_response(error)
    }
}

/// Trait for implementing different response converter implementations
/// the default is [TextErrorResponse]
pub trait IntoHttpErrorResponse {
    /// Handles converting the error into an HTTP response
    fn into_response(error: Box<dyn HttpError>) -> Response;
}

/// Creates HTTP errors responses where the "reason" is provided as
/// the text contents of the response and the status is used as the
/// HTTP status
pub struct TextErrorResponse;

impl IntoHttpErrorResponse for TextErrorResponse {
    fn into_response(error: Box<dyn HttpError>) -> Response {
        (error.status(), error.reason()).into_response()
    }
}

/// This trait should be implemented by error types that can be used
/// as HTTP error responses
pub trait HttpError: Error + Send + Sync + 'static {
    /// Handles logging the error when its translated into an HTTP error response
    ///
    /// Default implementation logs both the [Display] and [Debug] variants
    /// of the error
    #[cfg(feature = "log")]
    fn log(&self) {
        log::error!("{self}: {self:?}");
    }

    /// Handles determining the HTTP status code that should be used
    /// for the HTTP response
    ///
    /// Defaults to [StatusCode::INTERNAL_SERVER_ERROR]
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Handles creating the error response "reason" text that
    /// is included in the error
    fn reason(&self) -> String {
        self.to_string()
    }

    /// Provides the full type name for the actual error type thats been
    /// erased by dynamic typing (For better error source clarity) used by
    /// the [Debug] implementation of [DynHttpError]
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Allow conversion from implementors of [HttpError] into a [DynHttpError]
impl<E, I> From<E> for DynHttpError<I>
where
    E: HttpError,
    I: IntoHttpErrorResponse,
{
    fn from(value: E) -> Self {
        DynHttpError {
            inner: Box::new(value),
            _marker: PhantomData,
        }
    }
}
