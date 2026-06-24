# validator

For full patterns, prefer [`../boundary-defense.md`](../boundary-defense.md).
This file covers crate-specific defaults only.

Use `validator` on DTOs when the project already uses derive-based request validation.

Still convert validated DTOs into domain newtypes. The validation derive checks the DTO boundary; the domain constructor preserves invariants for every other construction path.

```rust
#[derive(serde::Deserialize, validator::Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    email: String,
}
```
