# Gradual Kamae Adoption

## Default Stance

Apply Kamae to new code paths first. Tighten existing code where you already
touch it for a feature or bugfix. Do not block delivery on a full-domain rewrite.

When legacy conventions conflict, follow the local convention for untouched code
and document the new boundary clearly where old and new meet.

## Recognize Legacy Shapes

Common starting points in Rust server codebases:

- anemic structs with free functions or service modules
- ORM row types used as domain entities
- `String` IDs and status strings instead of newtypes
- `anyhow` or `unwrap` through business logic
- handlers that call SQL or HTTP directly

These are migration sources, not failures. Pick the smallest change that removes
the next likely bug.

## Adoption Ladder

Move one rung at a time. Each step should be reviewable on its own.

| Step | Change | Typical touch points | Risk |
| --- | --- | --- | --- |
| 0. Boundary only | DTO/row -> `TryFrom` for new endpoints or consumers | handlers, message consumers | Low |
| 1. IDs and value objects | `RequestId`, `Money`, `OccurredAt` newtypes | models used by the changed flow | Low |
| 2. Domain errors | `thiserror` enums in new use cases | application layer | Low |
| 3. Typed state | state structs/enums for one important aggregate | domain module for that aggregate | Medium |
| 4. Ports | small repository traits behind the new use case | application + infrastructure | Medium |
| 5. Transactions and versions | atomic save, outbox, optimistic version checks | persistence adapter | Medium–High |

Skip steps only when the codebase already satisfies them.

## Strangler-Fig a Feature, Not the Whole Crate

For a legacy module:

1. Add a new use-case struct for the changed workflow.
2. Keep old entry points calling legacy code until the new path is proven.
3. Route new API versions, flags, or commands to the new use case.
4. Delete the old path after parity tests pass.

```text
legacy handler -> legacy service -> DB
new handler    -> AssignDriver use case -> port -> adapter -> DB
```

Prefer one aggregate or one endpoint per migration slice.

## Keep Diffs Reviewable

Practical rules for team rollout:

- Do not mix mechanical refactors with behavior changes in one PR when avoidable.
- Add tests at the new boundary before deleting the old path.
- Introduce newtypes and DTO conversion on touched fields only; widen later.
- Enable extra clippy/rustdoc checks on the crate or module you are hardening.
- Leave a short comment or ADR only when old and new semantics differ.

## When to Stop Climbing the Ladder

Not every struct needs a state machine or repository trait. Stay at the current
rung when:

- the code is stable, low-risk, and rarely changes
- the aggregate has no meaningful lifecycle or invariants
- the team cannot yet test persistence or concurrency behavior credibly

Raise the rung when bugs, compliance needs, or concurrency show the current
shape is too weak.

## Agent and Reviewer Expectations

When migrating:

- load [`adoption.md`](./adoption.md) for scope decisions
- load the target topic guide for the rung you are implementing
- use `kamae-rs-review` on the changed paths even if surrounding code is still legacy
- call out residual legacy risk explicitly instead of pretending the crate is fully migrated
