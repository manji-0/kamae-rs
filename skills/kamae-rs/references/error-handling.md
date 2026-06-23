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

## Async Use Cases and `Result`

In Rust server code, the idiomatic shape is `async fn -> Result<T, E>`, not
`Result<Future<_>, E>`. The future resolves to a `Result`; use `?` inside the
async body.

Keep layers distinct:

| Layer | Typical shape | Error type |
| --- | --- | --- |
| Domain transition | sync `fn` or consuming method | `DomainError` |
| Use case | `async fn` | `UseCaseError` with `#[from]` variants |
| Port / adapter | `async fn` in trait | `RepositoryError`, `ClientError`, ... |

Domain transitions should stay synchronous and pure when possible. Async belongs
in use cases and adapters that perform I/O.

```rust
pub async fn execute(
    &self,
    request_id: RequestId,
    driver: DriverAssignment,
) -> Result<(), AssignDriverError> {
    let waiting = self
        .resolver
        .find_waiting(&request_id)
        .await
        .map_err(AssignDriverError::Repository)?
        .ok_or(AssignDriverError::RequestNotFound { request_id })?;

    let transition = waiting
        .assign_driver(driver)
        .map_err(AssignDriverError::Domain)?;

    self.store
        .save_assigned(&transition.state, &transition.events)
        .await
        .map_err(AssignDriverError::Repository)?;

    Ok(())
}
```

Guidelines:

- Map infrastructure errors at the `.await` site with `map_err` or `#[from]`.
- Do not hold mutex guards or other locks across `.await` in use cases.
- Use `tokio::try_join!` or equivalent only in the use-case layer when parallel
  port calls are truly independent.
- Prefer typed retryable errors such as `ConcurrentModification` over stringly
  retry logic in handlers.

Do not introduce `ResultAsync`-style combinators unless the project already
standardizes on them. `?` plus layer-specific error enums is the default Kamae
approach in Rust.

## Chain Errors with `#[source]` and `#[from]`

Use `thiserror` source chaining so callers and observability tools can walk the
full failure path without string concatenation.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AssignDriverError {
    #[error("request not found: {request_id}")]
    RequestNotFound { request_id: RequestId },
    #[error("domain transition failed")]
    Domain(#[from] TaxiRequestError),
    #[error("persistence failed")]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database query failed")]
    Query(#[source] sqlx::Error),
    #[error("optimistic lock conflict on {aggregate_id}")]
    ConcurrentModification { aggregate_id: RequestId },
}
```

Guidelines:

- Wrap lower-layer errors with `#[from]` or explicit variants; avoid
  `format!("{e}")` as the only preserved context.
- Keep leaf variants semantic (`ConcurrentModification`, `RateLimited`) even
  when they wrap an infrastructure error.
- Do not attach PII to error messages or `Display` output; use domain IDs only
  (see [`logging-metrics.md`](./logging-metrics.md)).
- Prefer one authoritative error return per use-case path. Adapters map errors;
  domain code returns typed enums without logging every layer.

When integrating with structured logging, record the error once at the layer
that owns the operation and rely on `{error}` / `%error` formatting that prints
the full chain (see [Integrate Error Chains with Structured Logging](./logging-metrics.md#integrate-error-chains-with-structured-logging)).
