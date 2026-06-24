# garde

For full patterns, prefer [`../boundary-defense.md`](../boundary-defense.md).
This file covers crate-specific defaults only.

Use `garde` on DTOs when the project prefers derive-based validation with composable validation rules.

Keep domain constructors authoritative. Do not let a DTO validation rule become the only place a domain invariant exists.

## Common Combinations

| Stack | Pattern | Topic guide |
| --- | --- | --- |
| `garde` + `serde` + axum | `Json<Dto>` then `dto.validate()` then `Command::try_from(dto)` | [`boundary-defense.md`](../boundary-defense.md#http-extractors-axum--actix-web) |
| `garde` + `thiserror` | Map `garde` report to boundary error enum in adapter | [`error-handling.md`](../error-handling.md) |
| `garde` + leaf newtypes | DTO field validation + `TryFrom` for domain newtypes | [`domain-modeling.md`](../domain-modeling.md) |

`garde` validates the DTO shape; `TryFrom` remains authoritative for domain meaning (cross-field rules, tenant scope, ID semantics).
