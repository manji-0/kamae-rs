# Mutation Testing Checklist

Reference: [`../../kamae-rs/references/mutation-testing.md`](../../kamae-rs/references/mutation-testing.md).

## 17.1 Is mutation testing aimed at high-risk pure logic? - Low

Suggest `cargo mutants` when constructors, transitions, money/idempotency, or
boundary `TryFrom` paths changed and the ordinary suite looks complete but thin
on assertions. Do not require mutation jobs for glue, generated code, or flaky
I/O tests.

## 17.2 Are missed mutants fixed at the public API? - Medium

Flag follow-up tests that only assert against a private helper or a single
mutant string. Prefer public constructor/transition/use-case assertions (or
property laws) that would fail if the mutated behavior shipped.

## 17.3 Are skips and excludes justified? - Medium

Flag broad `#[mutants::skip]`, empty excludes, or config that silences whole
domain modules without a comment. Prefer path/`exclude_re` filters for
generated or `Debug` noise; keep domain entry points mutable.

## 17.4 Is CI incremental and risk-tied? - Low

Cross-check [`../../kamae-rs/references/ci-setup.md`](../../kamae-rs/references/ci-setup.md#optional-assertion-strength-checks).
Flag full-tree mutant gates on every PR for large workspaces, or required
mutant jobs without `--in-diff` / package filters / schedule split. Do not
require mutation CI where the team has not tuned `.cargo/mutants.toml`.
