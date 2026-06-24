# garde

For full patterns, prefer [`../boundary-defense.md`](../boundary-defense.md).
This file covers crate-specific defaults only.

Use `garde` on DTOs when the project prefers derive-based validation with composable validation rules.

Keep domain constructors authoritative. Do not let a DTO validation rule become the only place a domain invariant exists.
