# Development Guide

This document explains how to set up a local development environment for the
`kamae-rs` skill package and how to run the checks that keep the package valid.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) toolchain with `rustfmt`, `clippy`, and `rustdoc`
- Python 3 (used for package validation and the review probe)

No additional crates are required to edit the skill files. The Markdown
guides and checklists are plain text.

## Local Setup

Clone or navigate to the repository and run the package smoke test to confirm
that manifests, links, and scripts are intact:

```bash
python3 scripts/validate_package.py
```

If this passes, the skill package is structurally valid and can be loaded by a
Claude or Codex client that supports the manifest format.

## Running CI Checks Locally

The repository keeps skill content and tooling quality aligned. Run the same
checks that CI runs before pushing:

```bash
# Package validation
python3 scripts/validate_package.py

# Review probe smoke test
cargo run -q -p kamae-review-probe -- skills/kamae-rs/examples/taxi-request.rs --json

# Skill example crate (taxi-request)
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

See [`skills/kamae-rs/references/quality-gates.md`](./skills/kamae-rs/references/quality-gates.md)
for the canonical check commands and
[`skills/kamae-rs/references/ci-setup.md`](./skills/kamae-rs/references/ci-setup.md)
for workflow templates and when to extend the matrix.

For contributors working on this skill repository, read
[`skills/kamae-rs/references/development-setup.md`](./skills/kamae-rs/references/development-setup.md).

Application crates that follow the skill should also read
[`skills/kamae-rs/references/dev-environment.md`](./skills/kamae-rs/references/dev-environment.md)
for toolchain setup, crate layout, test layers, and the local check loop, and
[`skills/kamae-rs/references/local-validation.md`](./skills/kamae-rs/references/local-validation.md)
when bootstrapping from templates.

## Working on Skills

Skill files live under `skills/`:

- `skills/kamae-rs/SKILL.md` — dispatcher for implementation guidance
- `skills/kamae-rs/references/*.md` — topic guides
- `skills/kamae-rs-review/SKILL.md` — dispatcher for review procedures
- `skills/kamae-rs-review/checklist/*.md` — review checklists

When adding a new topic:

1. Add a reference file under `skills/kamae-rs/references/`.
2. Link it from `skills/kamae-rs/SKILL.md` in Step 2.
3. Add a matching checklist under `skills/kamae-rs-review/checklist/` if the
topic needs adversarial review coverage.
4. Link the checklist from `skills/kamae-rs-review/SKILL.md` in the routing
matrix and checklist order.
5. Run `python3 scripts/validate_package.py` before committing.

## Cross-References

Use directive comments when a document depends on another section or file:

```markdown
<!-- constrained-by ./skills/kamae-rs/references/ci-setup.md -->
```

These directives are checked by the package validator along with ordinary
Markdown links. Declare real dependencies only.

## Submitting Changes

Before opening a pull request, ensure:

- `python3 scripts/validate_package.py` passes.
- Any new Markdown links point to existing files or anchors.
- New skill frontmatter has `name` and `description` fields.
- Skill directory names and frontmatter `name` values match.

## License

This project is released under the MIT License. See [LICENSE](./LICENSE).
