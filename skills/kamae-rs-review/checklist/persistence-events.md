# Persistence and Events Checklist
Reference: [`../../kamae-rs/references/persistence-events.md`](../../kamae-rs/references/persistence-events.md).

## 6.1 Are state and domain events persisted atomically? - High

Flag use cases that save aggregate state and publish/insert events in separate operations without a transaction or outbox pattern.

## 6.2 Do repository traits express domain needs? - Medium

Flag large repository traits mirroring ORM CRUD rather than small interfaces required by the use case.

## 6.3 Are events generated outside persistence adapters? - Medium

Flag repositories that invent business events internally instead of persisting events supplied by the use-case/domain layer.

## 6.4 Do DB constraints mirror critical invariants? - Medium

Flag persistence that relies only on application checks for uniqueness, tenant ownership, non-negative balances, valid lifecycle states, or foreign-key existence when the database can enforce them.

## 6.5 Are retries and duplicate deliveries idempotent? - High

Flag commands, event handlers, outbox processors, and external calls that can double-apply money, inventory, lifecycle transitions, or notifications without idempotency keys or dedupe records.

## 6.6 Are persisted events versioned? - Medium

Flag event payloads without explicit event type/version, schema evolution strategy, or backward-compatible deserialization when events are stored or consumed asynchronously.
