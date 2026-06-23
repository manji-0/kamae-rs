---
name: crate-detection
description: Auto-detect Rust crates from Cargo.toml
applies-to: "*"
type: library-preference
alwaysApply: false
---

# Crate Detection
Read `Cargo.toml` and relevant workspace member manifests during implementation and review. Load crate guides only when their crate is present and relevant to the task.

Guide-backed detections:

- `thiserror` -> `references/crate-guides/thiserror.md`
- `anyhow` or `eyre` -> `references/crate-guides/anyhow.md`
- `serde` -> `references/crate-guides/serde.md`
- `validator` -> `references/crate-guides/validator.md`
- `garde` -> `references/crate-guides/garde.md`
- `nutype` -> `references/crate-guides/nutype.md`
- `secrecy` -> `references/crate-guides/secrecy.md`

Detection-only crates:

- Error: `snafu`
- Boundary/config: `serde_json`, `toml`, `config`
- Validation/newtype helpers: `derive_more`
- PII/secrets: `zeroize`
- Persistence: `sqlx`, `diesel`, `sea-orm`
- Async: `tokio`, `async-trait`

Detection-only means the crate should inform local code review or implementation context, but there is no plugin guide to load. Prefer existing project conventions and standard-library Rust patterns before adding dependencies.
