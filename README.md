# kamae-rs

_Kamae (構え) - a stance of readiness._

Rust skills for designing and reviewing robust server-side domain code. This is a Rust-oriented sibling of [`kamae-ts`](https://github.com/iwasa-kosui/kamae-ts): it keeps the same thin-skill, topic-guide, and review-checklist shape while translating the principles into Rust idioms.

## Provided Skills

### `kamae-rs`

Use when implementing, modifying, refactoring, or fixing Rust domain models, use cases, repositories, state transitions, boundary DTO parsing, typed errors, PII handling, or validation/review-adjacent code.

Core principles:

- Model domain meaning with enums, structs, private-field newtypes, and validated constructors.
- Make invalid state transitions fail at compile time where practical.
- Use `Result<T, E>` with domain-specific error enums.
- Convert external data through DTO/row/config structs before constructing domain types.
- Keep PII and secrets behind redacting wrappers.

### `kamae-rs-review`

Use during Rust code review. It walks severity-tagged checklist files for domain modeling, transitions, error handling, boundary validation, PII protection, persistence/events, and tests.

## Packaging

The package includes both Claude and Codex manifests:

- `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json` describe the Claude plugin package.
- `.codex-plugin/plugin.json` describes the Codex plugin package and points Codex at `./skills/`.

Run `python3 scripts/validate_package.py` before publishing or sharing a package archive. The smoke test validates JSON manifests, skill frontmatter, relative Markdown links, manifest skill paths, and crate-guide references.

## Customization
Rules live under `.claude/rules/`, `.codex/rules/`, user-level rule directories, or this repo's `rules/defaults/`. See [`rules/README.md`](./rules/README.md).

## Repository Layout

```text
skills/kamae-rs/          Implementation guidance
skills/kamae-rs-review/   Review procedure and checklist
rules/                    Project/user override format
```

## License

MIT
