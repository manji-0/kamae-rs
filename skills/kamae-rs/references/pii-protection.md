# Rust PII Protection

## Make Sensitive Data Hard to Log

Use a redacting wrapper or a typed value object by default for personal data.
Reserve `secrecy::SecretString` or `SecretBox<T>` for credentials, API keys,
tokens, passwords, and cryptographic material. General PII such as names or
email addresses usually belongs in `Redacted<T>` or a domain newtype with a
safe `Debug` implementation, not in `secrecy`.

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

## Classify Identifiers Before Logging

Field names such as `user_id` or `passenger_id` do not make an identifier safe.
Apply the rules in [`logging-metrics.md`](./logging-metrics.md#which-ids-belong-in-logs):

- **Safe by default**: opaque surrogate aggregate IDs, correlation IDs, internal
  job/transaction IDs, and bounded domain enums.
- **Never log**: secrets, government IDs, payment identifiers, contact identity,
  person descriptors, health data, precise location, and network tracking IDs.
- **Conditional**: person-linked IDs (`user_id`, `customer_id`, `patient_id`,
  `device_id`, partner references) only when the project documents them as opaque
  surrogates with safe `Display` / `Debug`.

Encode the decision in the type. If an ID must not appear in general logs,
prevent accidental emission through `Redacted<T>`, restricted formatting, or
adapter-only exposure.
