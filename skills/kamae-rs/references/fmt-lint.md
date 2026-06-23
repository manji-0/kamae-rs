# Rust Formatting and Lints

## Formatting Baseline

Run `cargo fmt` or `rustfmt` on touched Rust files before finishing a change. Formatting is not a style debate in Kamae: it keeps diffs reviewable so domain, boundary, PII, unsafe, and persistence changes are easier to inspect.

Do not hand-align code in ways `rustfmt` will undo. Prefer small helper functions or named value objects over formatting tricks that hide complex conditions.

## Clippy Baseline

Run `cargo clippy` for the relevant package or workspace when the project has a Rust crate available. Use the project's existing command if one exists.

Recommended default:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Adjust features, packages, or warning policy to match the repository. Do not introduce a stricter global lint policy casually in an unrelated change.

## Lints That Matter for Domain Safety

Pay special attention to lints and patterns that can hide invalid states or operational failures:

- `unwrap_used`, `expect_used`, `panic`, and unchecked indexing in domain/use-case code
- `todo`, `unimplemented`, and `unreachable` outside tests or proven invariants
- `large_enum_variant`, `result_large_err`, and needless cloning that may indicate awkward domain boundaries
- `float_cmp`, suspicious arithmetic, and lossy casts in money, quantity, duration, or unit code
- `wildcard_enum_match_arm` and broad `_` arms over domain enums
- `derive_partial_eq_without_eq`, broad `derive(Debug)`, and serialization derives on sensitive or invariant-bearing types
- `await_holding_lock`, detached tasks, and ignored `Result`s in use cases or adapters

Do not require every lint above to be globally enabled. Use them as review signals when they appear in touched code or local configuration.

## Suppression Rules

Keep `#[allow(...)]` as narrow as possible:

- Prefer item-level or expression-level allows over crate-level allows.
- Include a short reason when suppressing a lint that affects correctness, safety, PII, persistence, or error handling.
- Avoid blanket `#![allow(warnings)]`, `#![allow(clippy::all)]`, or broad module-level allows in production code.

Good:

```rust
#[allow(clippy::result_large_err, reason = "error enum preserves exhaustive domain handling")]
pub fn assign_driver(...) -> Result<..., AssignDriverError> { ... }
```

If the toolchain does not support `reason`, add a nearby comment.

## Generated and Third-Party Code

Do not force generated bindings, vendored code, or externally maintained snapshots through the same lint bar as domain code. Keep those files isolated and document the generation source.

Generated code may use broader allows, but safe wrappers around generated/FFI code still follow the unsafe boundary and boundary validation guidance.

## CI Expectations

Prefer CI jobs that run:

- `cargo fmt --check`
- `cargo clippy` with the repository's feature/package matrix
- tests relevant to domain constructors, transitions, boundary conversion, unsafe wrappers, and persistence behavior

When a project cannot run full workspace checks quickly, run the smallest package/feature set that covers the changed code and state the limitation.
