# Property-Based Tests Checklist

Reference: [`../../kamae-rs/references/property-based-tests.md`](../../kamae-rs/references/property-based-tests.md).

## 18.1 Do generators use public constructors? - High

Flag `proptest`/`quickcheck` strategies that build domain structs with raw
literals or private fields instead of `new`, `try_new`, or `TryFrom`.

## 18.2 Is each property a named invariant? - Medium

Flag property tests that only assert `is_ok()` or compare unstructured output
without stating the law (round trip, idempotence, rejection rule, etc.).

## 18.3 Are preconditions enforced with `prop_assume!`? - Medium

Flag properties that treat out-of-domain inputs as success or failure
ambiguously instead of discarding them explicitly.

## 18.4 Are illegal transitions tested for specific errors? - Medium

Cross-check [`state-transitions.md`](./state-transitions.md). Flag property
tests that only check `is_err()` on invalid transitions when callers depend on
the error variant.

## 18.5 Is non-deterministic I/O avoided inside properties? - High

Flag `proptest!` blocks that hit live databases, networks, or wall-clock time
without injected fakes or seeded clocks.

## 18.6 Are regression files committed for fixed shrink cases? - Low

Suggest `proptest-regressions` when a property found a subtle bug and the
minimal counterexample should not disappear silently.
