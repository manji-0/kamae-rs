# PII Protection Checklist
Reference: [`../../kamae-rs/references/pii-protection.md`](../../kamae-rs/references/pii-protection.md).

## 5.1 Are PII and secrets wrapped? - High

Flag bare `String`, `Vec<u8>`, or primitive fields carrying email, phone, address, names, government IDs, payment data, health data, IP addresses, precise location, tokens, or passwords.

Suggest `secrecy::SecretString`, `SecretBox<T>`, or a project-local redacting wrapper.

Do not require `SecretString` for every PII value. Non-secret identifiers such as display names, emails, or coarse IPs may use domain newtypes if `Debug`, logs, and serialization are redacted or intentionally exposed.

## 5.2 Can Debug or logs expose sensitive data? - High

Flag `#[derive(Debug)]`, `tracing` fields, formatted errors, or logs that include raw sensitive values.

Also check metrics, span attributes, audit events, panic messages, and validation errors for raw PII or secrets.

## 5.3 Is plaintext exposure narrow and named? - Medium

Flag broad getters such as `email(&self) -> &str` for sensitive values. Suggest adapter-specific exposure methods or wrappers.

## 5.4 Is observability redacted by default? - High

Flag logging/metrics helpers that accept arbitrary domain objects or DTOs without redaction policy, allowlist fields, or explicit safe display wrappers.
