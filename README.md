# Axum-dyn-error

Dynamic error handling for Axum HTTP handlers

```toml
[dependencies]
axum-dyn-error = "0.1"
```

Only supports axum `v0.6`

This crate provides foundational logic for creating and handling dynamic HTTP errors.

Implementing the `HttpError` trait on errors allows you to customize how the errors and 
displayed in error responses.

I recommend using `thiserror` for defining your user facing error types.

In order to use the dynamic error handling you should replace your [Result] return types with
the [HttpResult] type from `axum_dyn_error`.

```rust

use axum_dyn_error::{HttpResult, HttpError, StatusCode};
use thiserror::Error;
use axum::{extract::Path, Json};

#[derive(Debug, Error)]
pub enum ExampleError {
    #[error("User not found")]
    MissingUser,
    #[error("Username was invalid")]
    InvalidUsername
}

impl HttpError for ExampleError {

    /// Customize the HTTP status code
    fn status(&self) -> StatusCode {
        match self {
            ExampleError::MissingUser => StatusCode::NOT_FOUND,
            ExampleError::InvalidUsername => StatusCode::BAD_REQUEST
        }
    }
}

/// Dummy structure representing a user
pub struct User;

/// Mock function for finding a user by id
pub async fn get_user_by_id(user_id: u32) -> Option<User> { unimplemented!() }

/// Example handler
pub async fn example_handler(
    Path(user_id): Path<u32>
) -> HttpResult<Json<User>> {
    let user = get_user_by_id(user_id)
        .await
        .ok_or(ExampleError::MissingUser)?;

    Ok(Json(user))
}

```

## Anyhow support

Axum-dyn-error supports `anyhow` errors through the `anyhow` feature flag, by default the
`hide-anyhow` feature flag is enabled which prevents the anyhow error message from being
included in the error response instead responding with "Server error".

```rust

use axum_dyn_error::{HttpResult, HttpError, StatusCode};
use axum::{extract::Path, Json};
use anyhow::anyhow;

/// Dummy structure representing a user
pub struct User;

/// Mock function for finding a user by id
pub async fn get_user_by_id(user_id: u32) -> Option<User> { unimplemented!() }

/// Example handler
pub async fn example_handler(
    Path(user_id): Path<u32>
) -> HttpResult<Json<User>> {
    let user = get_user_by_id(user_id)
        .await
        .ok_or(anyhow!("Missing user"))?;

    Ok(Json(user))
}
```

Using [AnyhowStatusExt] the anyhow error types can have an HTTP status code associated with them, by
default anyhow errors just use "500 Internal server error":

```rust

use axum_dyn_error::{HttpResult, HttpError, StatusCode, anyhow::AnyhowStatusExt};
use axum::{extract::Path, Json};
use anyhow::anyhow;

/// Dummy structure representing a user
pub struct User;

/// Mock function for finding a user by id
pub async fn get_user_by_id(user_id: u32) -> Option<User> { unimplemented!() }

/// Example handler
pub async fn example_handler(
    Path(user_id): Path<u32>
) -> HttpResult<Json<User>> {
    let user = get_user_by_id(user_id)
        .await
        .ok_or(
            anyhow!("Missing user")
                .status(StatusCode::NOT_FOUND)
        )?;

    Ok(Json(user))
}
```

## Custom response

By default the responses generated from the errors use the "reason" as a text response
body. You can change this by create a structure and implementing `IntoHttpErrorResponse`
on that structure:

```rust

use axum_dyn_error::{HttpResult, HttpError, IntoHttpErrorResponse};
use axum::response::{Response, IntoResponse};

pub struct CustomErrorResponse;

impl IntoHttpErrorResponse for CustomErrorResponse {
    fn into_response(error: Box<dyn HttpError>) -> Response {
        // Your logic to create the response from the error example:
        (error.status(), error.reason()).into_response()
    }
}

// You can then alias the HttpResult type
pub type MyHttpResult<T> = HttpResult<T, CustomErrorResponse>;

```

### Crate Features

The default features are `["log", "hide-anyhow"]`

| Feature         | Description                                                                          |
| --------------- | ------------------------------------------------------------------------------------ |
| **log**         | Logs errors that are created using `log::error!`                                     |
| **anyhow**      | Adds support for handling `anyhow` error types                                       |
| **hide-anyhow** | Replaces anyhow error messages in HTTP responses with a generic server error message |
