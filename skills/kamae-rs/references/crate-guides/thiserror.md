# thiserror

For full patterns, prefer [`../error-handling.md`](../error-handling.md). This
file covers crate-specific defaults only.

Use `thiserror` for domain-specific error enums when the crate already depends
on it or when introducing a small, conventional error derive is acceptable.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid request id")]
    InvalidRequestId,
}
```

Keep variants semantic. Avoid catch-all variants such as `Other(String)` in domain errors unless they wrap an infrastructure failure at an application boundary.
