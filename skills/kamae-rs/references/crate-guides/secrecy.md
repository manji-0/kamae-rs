# secrecy

Use `secrecy` for credentials and other secrets that must not appear in `Debug`
output or linger in memory longer than necessary. Prefer `Redacted<T>` or domain
newtypes with custom `Debug` for personal data (see `pii-protection.md`).

Store secrets as `SecretString` or project-specific wrappers around
`SecretBox`. Expose values only through narrow adapter functions via
`ExposeSecret`.

Avoid including exposed secret values in error variants.
