# Rust State Transitions

## Constrain Transitions by Source Type

When only one state may transition, accept that specific state instead of a wide enum.

```rust
pub struct WaitingRequest {
    request_id: RequestId,
    passenger_id: PassengerId,
}

pub struct EnRouteRequest {
    request_id: RequestId,
    passenger_id: PassengerId,
    driver_id: DriverId,
}

impl WaitingRequest {
    pub fn assign_driver(self, driver_id: DriverId) -> EnRouteRequest {
        EnRouteRequest {
            request_id: self.request_id,
            passenger_id: self.passenger_id,
            driver_id,
        }
    }
}
```

This makes invalid source states fail at compile time.

Keep a transition infallible only when every precondition is encoded in the
input types. If any rule depends on data not represented by the source state or
argument types, return a domain error:

```rust
pub enum DomainError {
    DriverCannotServeAccessibilityRequest,
}

impl WaitingRequest {
    pub fn assign_driver(
        self,
        driver: DriverAssignment,
    ) -> Result<Transition<EnRouteRequest, TaxiRequestEvent>, DomainError> {
        if self.requires_accessible_vehicle && !driver.accepts_accessibility_requests {
            return Err(DomainError::DriverCannotServeAccessibilityRequest);
        }

        Ok(Transition {
            state: EnRouteRequest {
                request_id: self.request_id,
                passenger_id: self.passenger_id,
                driver_id: driver.driver_id,
            },
            events: vec![],
        })
    }
}
```

Do not hide this behind `panic!`, `unwrap()`, or comments such as "caller must
check first". If the compiler cannot enforce the precondition, the transition
signature should show that failure is possible.

## Use Enums at Boundaries

Use an aggregate enum when callers need to store, load, or branch over all possible states.

```rust
pub enum TaxiRequest {
    Waiting(WaitingRequest),
    EnRoute(EnRouteRequest),
    InTrip(InTripRequest),
    Completed(CompletedRequest),
    Cancelled(CancelledRequest),
}
```

Use exhaustive `match` arms. Avoid `_` in domain matches unless the fallback is truly invariant to every future variant.

Keep aggregate boundaries explicit: load and save the request aggregate as a
whole, and keep cross-aggregate references as IDs or snapshots instead of
borrowing mutable state from another aggregate. A transition should protect the
invariants owned by its aggregate and call out to a use case or policy when it
needs facts owned elsewhere.

## Return Transition Outcomes Explicitly

When a transition emits events, return an outcome struct rather than mutating hidden state.

```rust
pub struct Transition<TState, TEvent> {
    pub state: TState,
    pub events: Vec<TEvent>,
}
```

Prefer taking `self` by value for state-consuming transitions. Borrow only when the original state must remain available.
