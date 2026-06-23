# Rust Error Handling

## Use Domain-Specific Error Enums

Use `Result<T, E>` and specific error enums in domain and use-case code.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AssignDriverError {
    #[error("request not found: {request_id}")]
    RequestNotFound { request_id: RequestId },
    #[error("request is not waiting")]
    InvalidState,
    #[error("driver is not available: {driver_id}")]
    DriverNotAvailable { driver_id: DriverId },
}
```

Avoid returning `anyhow::Error`, `Box<dyn Error>`, or `String` from domain functions. Those are acceptable near application edges where errors are reported or logged.

## Avoid Panics in Domain Code

Flag or avoid `panic!`, `todo!`, `unimplemented!`, `unwrap()`, and `expect()` in domain and use-case code. Use typed errors or test-only helpers instead.

Acceptable exceptions:

- Tests and fixtures
- Truly unreachable branches guarded by exhaustive domain reasoning
- Process startup configuration failures where crashing is the intended behavior

## Convert Infrastructure Errors Deliberately

Map repository and adapter errors into use-case errors at the boundary between infrastructure and application logic.

```rust
let request = repository
    .find_by_id(&request_id)
    .await
    .map_err(AssignDriverError::Repository)?;
```

Do not let low-level crate error types become the public error contract of a domain use case unless that is an explicit project convention.
