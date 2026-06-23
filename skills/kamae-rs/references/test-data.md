# Rust Test Data

## Use Constructors in Fixtures

Build fixtures through the same constructors or test builders as production code. Avoid raw struct literals that bypass private invariants.

```rust
fn request_id(value: &str) -> RequestId {
    RequestId::new(value.to_owned()).expect("fixture request id is valid")
}
```

Using `expect` in test fixture helpers is acceptable when the message states the fixture invariant.

## Test Compile-Time State Safety

For important state-machine guarantees, add compile-fail tests with `trybuild` when the project already uses it or when the invariant is central enough to justify the dependency.

Use normal unit tests for successful transitions, error mapping, DTO conversion, and PII redaction behavior.

## Property-Based Tests

For generator design, state-machine laws, shrinking rules, regression files, and
CI budget, see [`property-based-tests.md`](./property-based-tests.md).

Use property-based tests when an invariant should hold across many inputs:
value-object constructors, parser/formatter round trips, state-machine
transition laws, money arithmetic, unit conversions, and timestamp boundary
rules.

Keep generated data flowing through public constructors. If a generator creates
raw fields directly, it can accidentally test a world that production code
cannot construct.
