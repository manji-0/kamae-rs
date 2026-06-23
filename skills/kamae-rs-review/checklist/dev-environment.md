# Development Environment Checklist

Reference: [`../../kamae-rs/references/dev-environment.md`](../../kamae-rs/references/dev-environment.md).

## 19.1 Is domain code free of I/O dependencies? - High

Flag `domain` crates or modules that depend on `sqlx`, `axum`, `tonic`, or other
infrastructure crates when the team claims a Kamae-style split.

## 19.2 Can domain and use-case tests run without Docker? - Medium

Flag workflows where basic transition or use-case tests require live databases
or external services when fake ports would suffice.

## 19.3 Are fixtures built through constructors? - Medium

Cross-check [`tests.md`](./tests.md). Flag test helpers that bypass invariants
with public field literals or raw ORM rows in domain/use-case tests.

## 19.4 Is a documented local check loop available? - Low

Flag projects adopting Kamae conventions without a fast path and full pre-push
command list aligned with [`ci-setup.md`](../../kamae-rs/references/ci-setup.md).

## 19.5 Are secrets and PII kept out of committed env files? - High

Cross-check [`pii-protection.md`](./pii-protection.md). Flag committed `.env`
files, real credentials in examples, or local setup docs that encourage logging
raw PII for debugging.

## 19.6 Does test layout match crate boundaries? - Medium

Flag domain tests that pull in HTTP servers or DB pools directly instead of
testing through fakes at the use-case layer or adapters at the infrastructure
layer.
