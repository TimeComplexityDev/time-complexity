# 15 — Refactor: unify error types across handlers

**What to build:** HTTP handlers currently use three different error return types:
- `(StatusCode, &'static str)` — auth middleware, pair_handler
- `(StatusCode, String)` — start_handler, stop_handler, set_params_handler
- `Result<_, String>` — capture functions

This creates friction: adding a new handler requires deciding which error type to use, and changing error formatting touches every handler signature.

**Blocked by:** None

**Labels:** ready-for-agent

**Status:** pending

## Proposed design

Define a single application error type:

```rust
enum AppError {
    BadRequest(&'static str),
    Conflict(&'static str),
    Unauthorized,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m).into_response(),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m).into_response(),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m).into_response(),
        }
    }
}
```

All handlers return `Result<Json<Value>, AppError>`. The `?` operator works with `String` errors via a `From<String>` impl.

## Changes

- Define `AppError` in `main.rs` (or a new `error.rs` module)
- Implement `IntoResponse` for `AppError`
- Update all handler signatures
- Remove `error_response` helper function
- Remove `Box::leak` workaround if any remain

**Files:** `apps/local-bridge/src/main.rs` (or new `apps/local-bridge/src/error.rs`)