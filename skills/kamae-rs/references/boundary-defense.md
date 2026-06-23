# Rust Boundary Defense

## Treat Deserialization as Shape Parsing Only

`serde` proves that data has the requested shape, not that it satisfies domain meaning. Deserialize into DTOs first, then convert into domain types with `TryFrom`.

```rust
#[derive(serde::Deserialize)]
pub struct CreateRequestDto {
    passenger_id: String,
}

impl TryFrom<CreateRequestDto> for CreateRequestCommand {
    type Error = CreateRequestError;

    fn try_from(dto: CreateRequestDto) -> Result<Self, Self::Error> {
        Ok(Self {
            passenger_id: PassengerId::new(dto.passenger_id)?,
        })
    }
}
```

## Validate Every External Boundary

Apply DTO -> domain conversion for:

- HTTP and RPC requests
- DB rows and query results
- Queue messages and webhooks
- Files, env vars, and config
- CLI arguments

Do not directly construct domain types from raw `String`, `Value`, or DB row fields unless the constructor validates invariants.

## Keep API, DB, and Domain Types Separate

Do not add `Serialize`, `Deserialize`, `sqlx::FromRow`, or Diesel derives to domain entities by default. Use DTO/row structs when the external representation differs or can bypass invariants.

Exceptions are reasonable for small internal tools or truly invariant-free value objects; state the reason when it matters.
