# Rust Unsafe Boundaries

## Default Stance

Keep `unsafe` out of domain logic. Domain entities, value objects, state transitions, use cases, DTO conversion, PII redaction, and repository traits should normally be safe Rust.

Use `unsafe` only when the requirement cannot be expressed with safe Rust at acceptable cost:

- FFI or OS/runtime integration
- Implementing a safe abstraction over raw pointers, memory layout, or initialization
- Interacting with crates whose contract requires unsafe calls
- Measured low-level performance work after a safe design has proven insufficient

Do not use `unsafe` to bypass ownership, validation, constructors, privacy, serde conversion, tenant checks, or error handling.

## Contain Unsafe Behind Safe APIs

Keep `unsafe` blocks as small as possible and place them in adapter or infrastructure modules, not in core domain modules.

Expose a safe API that enforces every precondition before entering `unsafe`:

```rust
pub struct NonEmptyBytes(Vec<u8>);

impl NonEmptyBytes {
    pub fn new(bytes: Vec<u8>) -> Result<Self, NonEmptyBytesError> {
        if bytes.is_empty() {
            return Err(NonEmptyBytesError::Empty);
        }
        Ok(Self(bytes))
    }

    pub fn first_byte(&self) -> u8 {
        // SAFETY: `NonEmptyBytes::new` rejects empty vectors and the field is private,
        // so every `NonEmptyBytes` value contains at least one byte.
        unsafe { *self.0.get_unchecked(0) }
    }
}
```

If the caller must uphold preconditions, make that explicit with an `unsafe fn` and document the contract in a `# Safety` section. Prefer safe functions when the module can check the precondition itself.

## Safety Comments

Every `unsafe` block, `unsafe fn`, `unsafe trait`, and `unsafe impl` should explain:

- What invariant makes the operation sound
- Where that invariant is established
- Why aliases, lifetimes, initialization, alignment, and bounds are valid
- How the invariant is preserved after future mutation or refactoring

Avoid comments that merely restate the operation, such as "dereference pointer". The comment must justify soundness.

## Preserve Domain Boundaries

Unsafe code must not create domain values by bypassing constructors or validation. Convert raw data into DTOs/rows first, then use the same `TryFrom`, `FromStr`, or constructor path as safe code.

Unsafe code must not expose PII through `Debug`, logs, panic messages, FFI callbacks, metrics labels, or raw memory dumps. Wrap or redact sensitive data before it crosses the unsafe boundary.

## Review and Testing

For unsafe boundary changes, add focused tests around the safe wrapper:

- Normal and boundary inputs
- Constructor rejection paths
- Mutation paths that preserve the safety invariant
- FFI error paths and null/invalid handles where applicable

When possible, run Miri, sanitizer builds, fuzz/property tests, or crate-specific safety tests for unsafe-heavy code. Do not require these tools for every domain change, but recommend them when the unsafe block owns memory, pointer aliasing, initialization, or FFI lifetime contracts.
