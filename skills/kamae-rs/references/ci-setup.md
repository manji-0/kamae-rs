# Rust CI Setup

## Default Stance

CI should enforce the same safety signals reviewers depend on: formatting, linting, tests, rustdoc, and package-specific validation. Keep the default pipeline boring and fast; add heavier checks only where their risk reduction is real.

Use the repository's existing commands first. If no CI exists, start with the smallest workflow that covers changed Rust domain code.

## Minimum Rust Checks

For a Rust crate or workspace, prefer these jobs:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo doc --no-deps --all-features
```

Adjust `--all-features`, packages, or warning policy when the project has a known feature matrix. Do not introduce `-D warnings` across a legacy workspace unless the team is ready to fix existing warnings.

For this skill package, also run:

```bash
python3 scripts/validate_package.py
python3 scripts/review_probe.py skills/kamae-rs/examples/taxi-request.rs --json
```

## GitHub Actions Example

Use this as a starting point, not as a universal template:

```yaml
name: ci

on:
  pull_request:
  push:
    branches: [main]

jobs:
  package:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - name: Validate skill package
        run: python3 scripts/validate_package.py
      - name: Smoke review probe
        run: python3 scripts/review_probe.py skills/kamae-rs/examples/taxi-request.rs --json

  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - name: Install Rust toolchain
        run: rustup toolchain install stable --profile minimal --component clippy rustfmt
      - name: Format
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      - name: Test
        run: cargo test --all-targets --all-features
      - name: Docs
        run: RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

If the repository has no `Cargo.toml`, omit the `rust` job and keep package validation jobs.

## Matrix Strategy

Use a matrix when domain behavior changes across:

- feature flags
- crates in a workspace
- MSRV and stable Rust
- database adapters or persistence backends
- target OS or architecture for FFI/unsafe code

Keep expensive matrix entries scheduled or manually triggered unless the risk justifies every pull request paying the cost.

## Unsafe and Security Checks

For unsafe-heavy crates, FFI wrappers, or memory-layout code, consider adding one or more optional jobs:

- `cargo miri test` for unsafe soundness and undefined behavior checks
- sanitizer builds for memory/thread issues
- fuzz/property tests for parsers, boundary conversion, and unsafe wrappers
- `cargo deny` or equivalent dependency policy checks
- secret scanning and dependency audit jobs when the repository handles credentials or PII

Do not require these jobs for every application crate by default. Tie them to risk: unsafe ownership, raw pointers, FFI lifetimes, parser trust boundaries, or compliance-sensitive data.

## CI Review Rules

When adding or reviewing CI:

- Keep checks named after the risk they guard, not after vague quality labels.
- Fail fast for formatting and package validation.
- Run tests after linting only when lint output is likely to be actionable.
- Cache build artifacts only if the cache key is specific enough to avoid stale feature/toolchain results.
- Do not hide failing checks behind `continue-on-error` unless the workflow clearly marks them as advisory.
- Protect `main` with required checks that match the review policy.

## Local Parity

Document a local command that approximates CI. Reviewers should be able to run the same core checks before pushing:

```bash
python3 scripts/validate_package.py
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

If full parity is too slow, document the fast path and the full path separately.
See [`dev-environment.md`](./dev-environment.md) for toolchain setup, crate
layout, test layers, fake ports, and the recommended local check loop.
