# Rust Mutation Testing

<!-- constrained-by ./quality-gates.md#tests -->
<!-- constrained-by ./property-based-tests.md -->
<!-- constrained-by ./ci-setup.md#optional-assertion-strength-checks -->

> **When to read:** Strengthening tests for domain constructors, transitions,
> boundary conversion, or other high-risk pure logic after ordinary and
> property tests already pass.
> **Related:** [`quality-gates.md`](./quality-gates.md),
> [`property-based-tests.md`](./property-based-tests.md),
> [`ci-setup.md`](./ci-setup.md#optional-assertion-strength-checks),
> [`test-data.md`](./test-data.md).

## What Mutation Testing Checks

Mutation testing asks a different question than coverage or property tests:
**if this line of production code were wrong, would any test fail?**

`cargo-mutants` injects small, deliberate bugs into source (for example
replacing a function body with `Ok(())`, changing a comparison, or deleting a
match arm), then runs the test suite. Outcomes:

| Outcome | Meaning | Action |
| --- | --- | --- |
| **caught** | A test failed under the mutant | Good signal that behavior is asserted |
| **missed** | Tests still passed | Likely gap — strengthen tests or skip with reason |
| **unviable** | Mutant does not compile | No coverage signal; usually ignore |
| **timeout** | Suite hung or ran too long | Investigate loops/retries; skip or fix |

Line or branch coverage only shows that code was *executed*. Mutation testing
shows whether tests *notice* wrong behavior. Use it as a probe on trusted,
deterministic suites — not as a substitute for writing clear domain tests.

## When Mutation Tests Earn Their Cost

<!-- constrained-by #what-mutation-testing-checks -->
<!-- constrained-by ./property-based-tests.md#when-property-tests-earn-their-cost -->

Run mutation testing when the cost of a silent wrong change is high and the
suite is already green and non-flaky.

Good targets:

- value-object constructors and validation predicates
- state-transition functions and illegal-transition errors
- money, quantity, and idempotency logic
- DTO `TryFrom` / boundary mapping that must reject bad input
- PII redaction and safe `Display`/`Debug` contracts
- safe wrappers that preserve invariants after refactoring

Prefer not to spend mutation budget on:

- pure glue, logging, or metrics side effects
- generated code, vendored bindings, or `Debug`/`Display` boilerplate (unless
  redaction is the contract)
- flaky integration tests or live I/O (fix those first)
- large UI or infrastructure trees where most mutants are noise

**How this relates to property tests:** property tests broaden *inputs*;
mutation tests deepen *assertion strength*. A suite can have high `proptest`
coverage and still miss a mutant that turns a transition into a no-op.
Use both on the same high-value domain modules when risk justifies it.

## Prefer `cargo-mutants`

For Rust domain crates, use [`cargo-mutants`](https://mutants.rs/) as the
default mutation tool. It works with `cargo test` (and optionally
`cargo nextest`), needs no source annotations to start, and integrates cleanly
with GitHub Actions.

Install locally:

```bash
cargo install --locked cargo-mutants
# or, for CI binaries: taiki-e/install-action with tool: cargo-mutants
```

Add `mutants.out/` to `.gitignore` (the skill gitignore template includes it).
Do not commit mutation output unless you deliberately archive a CI artifact.

## Prerequisites

<!-- constrained-by ./quality-gates.md#tests -->

Before running mutants:

1. `cargo test` (or the package subset you will mutate) passes **reliably** in
   a clean tree — including when run from a copy of the workspace.
2. Flaky tests are fixed or excluded. Mutation runs amplify flakes into noise.
3. You know which crates or paths matter (usually `domain` / pure application
   logic, not every adapter).

If the baseline fails, `cargo-mutants` cannot tell you anything useful. Align
local checks with [`quality-gates.md`](./quality-gates.md#tests) first.

## Local Procedure

### 1. Start narrow

```bash
# Whole workspace (slow)
cargo mutants

# One package
cargo mutants -p domain

# Files you just changed
cargo mutants --file 'src/domain/**/*.rs'

# Mutants that touch a git diff (good local PR rehearsal)
git diff main...HEAD > /tmp/pr.diff
cargo mutants --in-diff /tmp/pr.diff
```

Use `--in-place` in CI to avoid copying the tree. Locally, the default copy
mode is safer for experimentation.

### 2. Read only actionable outcomes

By default, stdout highlights **missed** and **timeout**. Inspect details under
`mutants.out/` (diffs, per-mutant logs). Re-run a single interesting path with
`--file` after you add tests.

### 3. Fix at the right abstraction

When a mutant is missed:

1. Prefer a test through the **public domain API** that would break if the
   mutated code were wrong — not a private-function unit test tailored to the
   mutant.
2. If the gap is an invariant over many inputs, add or tighten a property test
   (see [`property-based-tests.md`](./property-based-tests.md)).
3. If the mutant is equivalent to correct code, or the function is intentionally
   untested glue, skip it with an explicit reason (next section).

Avoid writing assertions that only exist to kill one mutant string. Encode the
business rule the mutant violated.

#### Example: missed no-op transition

Suppose `cargo mutants` reports that replacing `assign_driver` with
`Ok(self.into())` is **missed**. A weak follow-up would assert on a private
helper. Prefer a public-API test:

```rust
#[test]
fn assign_driver_moves_waiting_to_en_route() {
    let waiting = WaitingRequest::new(request_id(), passenger_id());
    let outcome = waiting.assign_driver(driver_id()).expect("assign");
    assert!(matches!(outcome.state, EnRouteRequest { .. }));
    assert_eq!(outcome.state.driver_id(), driver_id());
}
```

That test fails if the transition becomes a no-op, without chasing the mutant
diff text.

## Scope, Filters, and Skips

Keep noise low so missed mutants stay actionable.

Start from the bundled template
[`../assets/templates/mutants.toml`](../assets/templates/mutants.toml)
(`.cargo/mutants.toml`):

```toml
exclude_globs = [
  "**/generated/**",
  "**/bin/**",
]
```

Skip a function when mutants are not meaningful. Prefer config excludes for
whole modules; use an attribute when the decision must sit next to the code.

The `cargo-mutants` **runner** is a tool (`cargo install` / CI install-action),
not a normal crate dependency. Add the tiny [`mutants`](https://docs.rs/mutants/)
helper crate only when you need `#[mutants::skip]` (regular `dependency`, or
`dev-dependency` if you always nest the attribute in `cfg_attr`):

```toml
[dependencies]
mutants = "0.0.3"
```

```rust
#[mutants::skip]
fn cache_warming_hint() {
    // Performance-only; behavior covered elsewhere or accepted risk.
}
```

Document skips in the adjacent comment or rustdoc. Blind skips defeat the
purpose of the gate. Prefer path/`exclude_re` filters in `.cargo/mutants.toml`
for generated code and `Debug` noise before sprinkling attributes on domain
entry points.

Package-level focus for domain work:

```bash
cargo mutants -p domain -- --test-threads=1   # only if tests share state
```

Prefer fixing shared-state flakiness over forcing single-threaded runs.

## Integrate with Existing Test Layers

<!-- constrained-by ./dev-environment.md#test-layers -->
<!-- constrained-by ./test-data.md -->

| Layer | Mutation testing role |
| --- | --- |
| Value object | Catch no-op constructors, always-`Ok` validators, swapped comparisons |
| Domain transition | Catch deleted arms, always-success transitions, ignored error variants |
| Use case | Catch missing port calls or skipped idempotency checks (with fakes) |
| Boundary DTO | Catch mappings that accept invalid payloads or drop fields silently |
| Property / example tests | Provide the failing assertions that *catch* mutants |

Mutation testing does not replace fixtures, `trybuild`, or property tests — it
audits whether those tests actually constrain behavior. See
[`dev-environment.md`](./dev-environment.md#test-layers) for which layer owns
which proof.

## CI Guide

<!-- constrained-by ./ci-setup.md#optional-assertion-strength-checks -->
<!-- constrained-by ./ci-setup.md#matrix-strategy -->

Mutation jobs are **optional assertion-strength checks**, not unsafe/security
probes. See [`ci-setup.md`](./ci-setup.md#optional-assertion-strength-checks).
Do not block every trivial PR on a full-tree mutant run.

### Recommended posture

| Mode | When | Command shape |
| --- | --- | --- |
| **PR incremental** | Default for teams adopting mutants | `--in-diff` against the base branch |
| **Scheduled / main full** | Nightly or weekly on `domain` | `cargo mutants -p domain --in-place` |
| **Sharded full** | Large trees that must finish in CI | `--shard i/n` + `--baseline=skip` after a green test job |

### Templates

Copy rather than paste YAML from this guide:

```bash
python3 path/to/kamae-rs/skills/kamae-rs/scripts/apply_templates.py \
  --target . --ci none --mutants
```

- Workflow: [`../assets/templates/github-ci-mutants.yml`](../assets/templates/github-ci-mutants.yml)
  → `.github/workflows/cargo-mutants.yml`
- Config: [`../assets/templates/mutants.toml`](../assets/templates/mutants.toml)
  → `.cargo/mutants.toml`

Adjust `-p domain` and `paths:` filters to your crates. Upload `mutants.out`
when the job fails.

### Full suite and branch protection

Run full suites after ordinary tests are green. Use `--baseline=skip` only when
a prior job already proved `cargo test` passes. Shard when wall-clock exceeds
budget ([sharding](https://mutants.rs/shards.html)).

- Keep format, clippy, and unit/integration tests **required**.
- Make mutation jobs required only after excludes and incremental runs are
  stable.
- Never hide missed mutants behind `continue-on-error` unless the workflow
  clearly marks the job as advisory.

### Local parity

```bash
git fetch origin main
git diff origin/main...HEAD > /tmp/pr.diff
cargo mutants --in-diff /tmp/pr.diff -p domain
```

Document the package list and `.cargo/mutants.toml` knobs next to the workflow
so reviewers can reproduce CI failures.

## Detection Hints

When reviewing or implementing domain crates:

- If `Cargo.toml` depends on `mutants`, or CI runs `cargo mutants`, load this
  guide with [`quality-gates.md`](./quality-gates.md) and the topic guide for
  the mutated code (modeling, transitions, boundaries, PII).
- Surviving mutants in constructors or transitions are review leads — treat
  them like missing tests for illegal states or silent success paths.
- Prefer fixing tests over broad `#[mutants::skip]` on domain entry points.

## Summary

<!-- derived-from #what-mutation-testing-checks -->
<!-- derived-from #when-mutation-tests-earn-their-cost -->
<!-- derived-from #local-procedure -->
<!-- derived-from #ci-guide -->

Mutation testing with `cargo-mutants` measures whether tests *detect* wrong
behavior, not merely whether code ran. Use it on high-value, deterministic
domain logic after the ordinary suite is green; kill missed mutants with
public-API or property tests; keep CI incremental on PRs and full runs on a
schedule unless the tree is small enough for every merge.
