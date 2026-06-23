---
name: kamae-rs-review
description: |
  Adversarial review of server-side Rust domain code for Kamae principles:
  explicit domain types, typed state transitions, Result-based domain errors,
  validated boundaries, PII redaction, and persistence/event consistency. Use
  when reviewing Rust pull requests, diffs, audits, or quality checks involving
  domain models, use cases, repositories, DTO conversion, or business logic.
  Skip frontend assets, build scripts, pure infrastructure, unsafe/performance
  tuning, and code unrelated to domain behavior.
---

# Kamae Rust Review
Review Rust code against the knowledge base in `../kamae-rs/`. Prioritize bugs, invalid states, data leaks, and missing tests over style.

## Step 0: Load Applicable Rules

Read matching rule files in priority order:

1. `.claude/rules/*.md` and `.codex/rules/*.md` in the project root
2. `~/.claude/rules/*.md` and `~/.codex/rules/*.md`
3. `../../rules/defaults/*.md` relative to this `SKILL.md`

Skip rules unless `applies-to` is `kamae-rs-review` or `*`. A `check-toggle` rule with `enabled: false` disables the named check. A `convention` rule changes review expectations.

## Review Procedure

1. Read [`../kamae-rs/SKILL.md`](../kamae-rs/SKILL.md).
2. Read `Cargo.toml` and relevant crate guides under `../kamae-rs/references/crate-guides/`.
3. Read the Rust files under review.
4. Choose checklist scope:
   - Full adversarial review: walk every checklist below in order.
   - Small/targeted diff: load only checklist files matched by the routing matrix, plus `tests.md` when behavior changes.
5. Report findings first, ordered by severity. Include `path:line`, risk, principle reference, evidence, and a concrete fix.

## Review Routing Matrix

| Diff signal | Load checklists |
| --- | --- |
| New/changed domain types, value objects, enums, constructors, mutators, monetary/time/unit fields | `domain-modeling.md`, `state-transitions.md`, `tests.md` |
| State-machine transitions, lifecycle/status changes, optimistic locking, command handlers | `state-transitions.md`, `persistence-events.md`, `tests.md` |
| `Result`, error enums, panics, `unwrap`/`expect`, infrastructure error mapping | `error-handling.md`, `tests.md` |
| HTTP/queue/CLI/config/DB input, DTOs, serde derives/defaults, row mapping | `boundary.md`, `domain-modeling.md`, `tests.md` |
| PII/secrets/tokens, logging, tracing, metrics, errors, `Debug`/`Display` | `pii-protection.md`, `tests.md` |
| Repositories, transactions, DB constraints, outbox/events, retries/idempotency | `persistence-events.md`, `state-transitions.md`, `tests.md` |
| Test-only helpers, builders, fixtures, compile-fail coverage | `tests.md` |

Use nearby checklists when a diff crosses concerns. Do not load unrelated files just to restate generic advice.

## Checklist Order

- [`checklist/domain-modeling.md`](./checklist/domain-modeling.md)
- [`checklist/state-transitions.md`](./checklist/state-transitions.md)
- [`checklist/error-handling.md`](./checklist/error-handling.md)
- [`checklist/boundary.md`](./checklist/boundary.md)
- [`checklist/pii-protection.md`](./checklist/pii-protection.md)
- [`checklist/persistence-events.md`](./checklist/persistence-events.md)
- [`checklist/tests.md`](./checklist/tests.md)

## Severity Classes

- High: likely runtime failure, impossible state admitted, unvalidated external data, or PII leak.
- Medium: weak domain contract, non-exhaustive error/state handling, persistence consistency risk.
- Low: maintainability, idiom, or test-quality issue that does not immediately compromise correctness.

Escalate when the diff touches external boundaries, authorization/tenant isolation, money, irreversible lifecycle transitions, persistence/event atomicity, secrets, or production observability. Downgrade when the risk is compile-time contained, test-only, startup-only, internal to a trusted adapter, or blocked by a nearby invariant not visible at the flagged line. Do not report a finding without evidence that a realistic caller can reach the bad state or leak.

Required evidence:

- Show the bypass path or missing guard, not only the smell.
- Name the invariant or domain rule being broken.
- Confirm whether existing constructors, validators, DB constraints, auth checks, or tests already cover it.
- Prefer "no issue" over speculative style findings.

If no issues are found, say so clearly and mention residual risk or test gaps.
