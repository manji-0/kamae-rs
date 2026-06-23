# secrecy

Use `secrecy` for secrets and PII values that must not appear in `Debug` output.

Store sensitive strings as `SecretString` or project-specific wrappers around it. Expose values only through narrow adapter functions that need plaintext.

Avoid including exposed secret values in error variants.
