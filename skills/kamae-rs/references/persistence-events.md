# Rust Persistence and Domain Events

## Separate Repositories by Responsibility

Use repository traits to express domain needs, not ORM convenience. Keep read and write interfaces small.

```rust
pub trait RequestResolver {
    async fn find_waiting(&self, id: &RequestId) -> Result<Option<WaitingRequest>, RepositoryError>;
}

pub trait RequestStore {
    async fn save_assigned(
        &self,
        state: &EnRouteRequest,
        events: &[DomainEvent],
    ) -> Result<(), RepositoryError>;
}
```

Prefer native `async fn` in traits for internal traits on Rust 1.75+ when
callers use static dispatch and do not need `dyn Trait`. Use `async_trait` when
the project must support an older MSRV, a framework requires trait objects, or
the repository is intentionally stored behind `Box<dyn RequestStore + Send +
Sync>`. The tradeoff is explicit: native traits avoid macro expansion and
boxing in the static-dispatch path; `async_trait` gives ergonomic dynamic
dispatch by boxing returned futures.

## Persist State and Events Atomically

When transitions emit domain events, save state changes and outbox rows in the same transaction. Avoid APIs that let callers save state and events in separate operations.

For aggregate roots, optimistic versioning, pessimistic locking, and use-case
transaction boundaries, see [`aggregate-transactions.md`](./aggregate-transactions.md).

## Keep Event Records Immutable

Model events as explicit structs or enums. Include identifiers, timestamp, aggregate id, event name/type, and payload. Generate events in the use-case/domain layer, not inside repository persistence code.

Use typed timestamps, money, and units in event payloads. For example, prefer
`OccurredAt`, `Money`, `DistanceMeters`, or `CurrencyCode` value objects over
bare `String`, `i64`, or `f64` fields. Event records are long-lived contracts;
make units and precision obvious at the type boundary.

## Expose Persisted Events as Streams When Needed

When read models, integrations, or operators subscribe to change feeds, expose
persisted events or outbox rows through `futures::Stream` ports rather than
pushing ad-hoc polling loops into use cases. See
[`stream-continuous-queries.md`](./stream-continuous-queries.md) for backpressure,
checkpoints, and projection idempotency.
