---
name: kamae-rs
description: |
  Kamae for Rust - robust server-side Rust domain design. Use when implementing,
  modifying, refactoring, or fixing Rust domain models, use cases, repositories,
  state transitions, error enums, Result-based workflows, boundary DTO parsing,
  validation, PII handling, safe wrappers around unsafe/FFI boundaries,
  rustfmt/clippy quality gates, rustdoc API contracts for domain code,
  CI setup for Rust domain checks, business logic, or review-adjacent remediation.
  Applies to server-side Rust crates, backend services, domain crates, and CLIs
  with business rules. Skip frontend assets, build scripts, pure infrastructure,
  low-level unsafe/performance tuning unrelated to domain boundaries, and code
  unrelated to domain logic.
---

# Kamae Rust

Use this skill as a thin dispatcher. Read only the topic and crate guide files relevant to the current task.

## Step 0: Load Applicable Rules

Before any other step, read matching rule files in priority order:

1. `.claude/rules/*.md` and `.codex/rules/*.md` in the project root
2. `~/.claude/rules/*.md` and `~/.codex/rules/*.md`
3. `../../rules/defaults/*.md` relative to this `SKILL.md`

For each rule:

- Read YAML frontmatter. Skip it unless `applies-to` is `kamae-rs` or `*`.
- Group by `name`. The first tier above wins over later tiers; within a tier, the lexicographically last filename wins.
- Apply surviving `library-preference`, `convention`, and `override` rules throughout the task.

## Step 1: Detect Rust Context

Read `Cargo.toml` and the workspace members relevant to the edited files. Note these dependencies if present. Crates with guide files load the guide only when relevant; detection-only crates inform local conventions but do not require a guide.

- Error: `thiserror`, `anyhow`, `eyre`; detection-only: `snafu`
- Boundary/serialization: `serde`; detection-only: `serde_json`, `toml`, `config`
- Validation/newtype: `validator`, `garde`, `nutype`; detection-only: `derive_more`
- PII/secrets: `secrecy`; detection-only: `zeroize`
- Logging/tracing/metrics: `tracing`, `log`, `metrics`; monitoring export base: `opentelemetry`; optional pull exporter: `prometheus`
- Detection-only persistence: `sqlx`, `diesel`, `sea-orm`
- Detection-only async: `tokio`, `async-trait`, `futures`, `tokio-stream`, `async-stream`
- Detection-only RPC/messaging: `tonic`, `prost`, `lapin`, `rdkafka`
- Detection-only resilience: `tower`, `governor`
- Detection-only testing: `proptest`, `quickcheck`, `proptest-regressions`, `trybuild`

If a dependency is relevant, load the matching file under [`references/crate-guides/`](./references/crate-guides/). If no crate guide matches, use standard-library Rust idioms before introducing a new dependency.

## Step 2: Load Topic Guides

Read only the topic file(s) needed for the task:

- Application Wiring: [`references/application-wiring.md`](./references/application-wiring.md)
- Aggregates and Transactions: [`references/aggregate-transactions.md`](./references/aggregate-transactions.md)
- Gradual Adoption: [`references/adoption.md`](./references/adoption.md)
- Domain Modeling: [`references/domain-modeling.md`](./references/domain-modeling.md)
- State Transitions: [`references/state-modeling.md`](./references/state-modeling.md)
- Error Handling: [`references/error-handling.md`](./references/error-handling.md)
- Boundary Defense: [`references/boundary-defense.md`](./references/boundary-defense.md)
- PII Protection: [`references/pii-protection.md`](./references/pii-protection.md)
- Logging and Metrics: [`references/logging-metrics.md`](./references/logging-metrics.md)
- Unsafe Boundaries: [`references/unsafe-boundaries.md`](./references/unsafe-boundaries.md)
- Formatting and Lints: [`references/fmt-lint.md`](./references/fmt-lint.md)
- Rustdoc Contracts: [`references/rustdoc.md`](./references/rustdoc.md)
- CI Setup: [`references/ci-setup.md`](./references/ci-setup.md)
- Persistence and Events: [`references/persistence-events.md`](./references/persistence-events.md)
- Streams and Continuous Queries: [`references/stream-continuous-queries.md`](./references/stream-continuous-queries.md)
- Domain Macros: [`references/domain-macros.md`](./references/domain-macros.md)
- Service Boundaries: [`references/service-boundaries.md`](./references/service-boundaries.md)
- Test Data: [`references/test-data.md`](./references/test-data.md)
- Property-Based Tests: [`references/property-based-tests.md`](./references/property-based-tests.md)

## Core Stance

Model invalid states and invalid transitions out of the type system where it is practical:

- Use enums, structs, newtypes, private fields, and `TryFrom`/`FromStr` constructors.
- Use `Result<T, E>` with domain-specific error enums in domain and use-case code.
- Avoid `panic!`, `unwrap()`, and `expect()` in domain code.
- Parse external data into DTOs first, then convert DTOs into domain types.
- Keep persistence models, API DTOs, and domain models separate unless the project has an explicit convention otherwise.
- Keep `unsafe` out of domain logic by default. When FFI, memory layout, or measured low-level performance requires it, hide it behind a small safe API with documented safety invariants.
- Keep `rustfmt` and `clippy` clean for touched Rust code. Treat lint suppressions as design decisions that need narrow scope and a reason.
- Document public domain APIs with rustdoc that states invariants, errors, state transitions, examples, and safety contracts where relevant.
- Keep CI aligned with the checks reviewers rely on: format, lint, tests, rustdoc, and optional unsafe/security probes.

These are strong defaults, not absolutes. If existing project conventions conflict, follow the convention and leave a brief explanation when the deviation affects domain safety.

## Examples

Read [`examples/taxi-request.rs`](./examples/taxi-request.rs) only when a concrete state-transition example would clarify the task. The example intentionally omits rustdoc; follow [`references/rustdoc.md`](./references/rustdoc.md) for production public APIs.
