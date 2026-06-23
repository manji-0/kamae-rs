# Error Handling Checklist
Reference: [`../../kamae-rs/references/error-handling.md`](../../kamae-rs/references/error-handling.md).

## 3.1 Are panics avoided in domain and use-case code? - High

Flag `panic!`, `todo!`, `unimplemented!`, `unwrap()`, and `expect()` outside tests, fixtures, startup code, or truly unreachable branches.

Do not flag fail-fast startup/configuration panics, test/fixture panics, migration assertions, or `expect` messages guarding invariants already proven in the same expression.

## 3.2 Are domain errors specific enums? - Medium

Flag `anyhow::Result`, `eyre::Result`, `Box<dyn Error>`, `String`, or opaque catch-all errors returned from domain constructors and use cases.

## 3.3 Are infrastructure errors converted intentionally? - Medium

Flag leaking `sqlx::Error`, `diesel::result::Error`, HTTP client errors, or config errors directly through public domain/use-case APIs.

## 3.4 Are error variants meaningful to callers? - Low

Flag vague variants such as `Other(String)` or `InvalidInput(String)` when callers need to branch exhaustively.
