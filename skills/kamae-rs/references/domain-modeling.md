# Rust Domain Modeling

## Represent Domain Concepts Explicitly

Use named structs, enums, and newtypes instead of primitive strings and numbers for semantically distinct values.

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(String);

impl RequestId {
    pub fn new(value: String) -> Result<Self, RequestIdError> {
        if value.trim().is_empty() {
            return Err(RequestIdError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Keep newtype fields private unless the value is deliberately transparent and has no invariant.

Model time, money, and units as explicit concepts. Prefer `OccurredAt`,
`ServiceDate`, `Money`, `CurrencyCode`, `DistanceMeters`, or
`DurationSeconds` over bare primitives whose unit, timezone, precision, or
rounding behavior is implicit. Avoid `f32`/`f64` for money.

## Prefer Enums for Variants and State

Use Rust enums for closed sets of states or domain alternatives. Prefer struct-like variants when each state carries different data.

```rust
pub enum TaxiRequest {
    Waiting(WaitingRequest),
    EnRoute(EnRouteRequest),
    InTrip(InTripRequest),
    Completed(CompletedRequest),
    Cancelled(CancelledRequest),
}
```

Use separate state structs when transitions should accept only a specific source state.

## Define Aggregate Boundaries

An aggregate owns the invariants that must change atomically. Put transition
methods on the state or aggregate that owns the rule, and reference other
aggregates by ID unless a use case has loaded a stable snapshot for a decision.

Avoid "god" aggregates that collect unrelated entities only to make access
convenient. Also avoid transitions that mutate two aggregate roots in memory and
then rely on callers to persist both correctly; use a use case plus explicit
domain events for cross-aggregate coordination.

## Keep Construction Honest

Use `new`, `try_new`, `TryFrom`, or `FromStr` to enforce invariants at construction time. Do not expose public fields that let callers bypass those invariants.

Accept direct struct literals only for simple data with no invariants or for test-only builders kept in test modules.

## Derive Traits Deliberately

Do not derive `Default` for invariant-bearing domain types unless there is a
real domain default. "Empty" IDs, zero money, or the first enum variant are
usually invalid or misleading defaults.

Derive `Clone` narrowly. It is usually fine for small immutable value objects
and DTOs, but broad `Clone` on aggregates and entities can hide ownership
mistakes or make stale copies easy to persist.

Do not derive unrestricted `Serialize` or `Deserialize` on domain types that
have private invariants. Use DTOs, row structs, or serde `try_from` on leaf
value objects so deserialization still runs validation.

## Keep Domain Models Separate

Avoid using the same struct for API JSON, DB rows, and domain entities. External shapes often contain optional or denormalized fields that should not leak into the domain model.

Use this flow:

```text
API/DB/env raw data -> DTO/row struct -> TryFrom -> domain type
```

## Organize by Concept

Prefer one domain concept per file or module: type, constructors, methods, and tests together. Avoid catch-all `types.rs` or `models.rs` modules that split types from behavior.
