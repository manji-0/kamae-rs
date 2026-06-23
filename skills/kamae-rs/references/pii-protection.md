# Rust PII Protection

## Make Sensitive Data Hard to Log

Use a redacting wrapper or a typed value object by default for personal data.
Use `secrecy::SecretString` or `SecretBox<T>` for credentials, API keys,
tokens, passwords, cryptographic material, or values whose normal workflows
need explicit plaintext exposure.

PII examples include names, email addresses, phone numbers, addresses, government IDs, payment identifiers, health data, IP addresses, and precise location data.

```rust
pub struct Redacted<T>(T);

pub struct Patient {
    id: PatientId,
    email: EmailAddress,
    diagnosis: Redacted<DiagnosisCode>,
}
```

Avoid deriving `Debug` on structs that contain raw PII. If `Debug` is needed,
redact sensitive fields manually or rely on a wrapper whose `Debug`
implementation redacts.

For credentials and secrets, prefer a secrecy type:

```rust
use secrecy::SecretString;

pub struct PaymentGatewayCredentials {
    api_key: SecretString,
}
```

## Keep Exposure Explicit

Expose sensitive values only at boundaries that genuinely need them, such as
email delivery, payment processors, encryption adapters, or audit export jobs.
Prefer methods named to communicate exposure:

```rust
pub fn expose_for_delivery(&self) -> &EmailAddress {
    &self.email
}
```

Never format sensitive values into domain errors or logs.
