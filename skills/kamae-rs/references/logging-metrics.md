# Rust Logging and Metrics

<!-- constrained-by ./pii-protection.md -->
<!-- constrained-by ./state-modeling.md -->
<!-- constrained-by ./error-handling.md -->

## Log with Domain Context

Every log entry should answer three questions: what happened, to which domain
object, and why it matters. Emit logs from use cases, application services, and
adapters rather than from deep inside domain invariants.

1. **Meaningful message**: describe the event or decision in domain terms, not
   the name of the function that emitted it. Prefer `"driver assigned to waiting
   request"` over `"assign_driver called"`.
2. **Domain object state**: include the identifiers, current state variant, and
   any values needed to understand the decision. Prefer structured fields over
   interpolated strings.
3. **Transition information**: when the operation's purpose is a state
   transition, log the source state, target state, and the command or event that
   triggered it.

```rust
#[derive(Clone, Debug)]
pub struct AssignDriverLog {
    request_id: RequestId,
    passenger_id: PassengerId,
    driver_id: DriverId,
    from: TaxiRequestState,
    to: TaxiRequestState,
    triggered_by: CommandId,
}

tracing::info!(
    request_id = %log.request_id,
    passenger_id = %log.passenger_id,
    driver_id = %log.driver_id,
    from = ?log.from,
    to = ?log.to,
    triggered_by = %log.triggered_by,
    "driver assigned to waiting request"
);
```

## Prefer Structured Logging

Use key-value fields instead of parsing meaning out of human-readable sentences.
Keep log templates stable so aggregators can group by message and filter by
field.

```rust
// Good: stable template, structured fields.
tracing::info!(
    request_id = %request_id,
    state = ?state,
    "request state persisted"
);

// Avoid: values baked into the message text.
tracing::info!("request {} persisted in state {:?}", request_id, state);
```

Choose log levels deliberately:

- `ERROR`: a domain invariant failed, a use case could not complete, or an
  infrastructure dependency is unhealthy. Include enough context to reproduce
  the failure without leaking secrets.
- `WARN`: a recoverable anomaly, such as a retryable timeout or an unexpected
  but handled edge case.
- `INFO`: a significant business event or lifecycle step.
- `DEBUG`: detailed state useful while diagnosing a specific problem. Guard
  expensive values with `tracing::debug!` so they are only evaluated at the
  matching level.

## Protect Logs from PII Leaks

Logs are long-lived and broadly accessible: treat them as a public boundary.
Follow the rules in [`pii-protection.md`](./pii-protection.md).

- Do not log raw names, emails, phone numbers, addresses, location, tokens, or
  credentials.
- Use newtypes and redacting wrappers so that accidentally deriving `Debug` or
  interpolating the value still produces a safe representation.
- When an identifier is sensitive, log a hash or opaque reference instead.

```rust
// Good: only non-sensitive identifiers and states appear in logs.
tracing::info!(
    request_id = %request_id,
    passenger_id = %passenger_id,
    state = ?state,
    "request transitioned to en-route"
);

// Avoid: a raw email would leak into log storage.
tracing::info!("notification sent to {}", email);
```

## Log State Transitions Explicitly

State transitions are central to domain behavior. Log them with both the before
and after state so traces, audit records, and incident investigations can
reconstruct the lifecycle.

When a transition emits events, include the event names or types, not the full
payload, unless the payload is already safe and useful for operations.

```rust
let outcome = waiting_request.assign_driver(driver)?;

tracing::info!(
    request_id = %outcome.state.request_id,
    from = "waiting",
    to = "en-route",
    events = ?outcome.events.iter().map(|e| e.name()).collect::<Vec<_>>(),
    "driver assignment completed"
);
```

Keep domain-level logging close to the use case that owns the transaction. Do
not scatter log statements inside every getter or validation helper.

## Keep Errors Actionable

Log domain errors with enough context to trace the failing path and the affected
objects. Reuse the structured identifiers from the surrounding use case rather
than creating ad-hoc labels.

```rust
match repository.find_by_id(&request_id).await {
    Ok(Some(request)) => request,
    Ok(None) => {
        tracing::warn!(request_id = %request_id, "request not found");
        return Err(AssignDriverError::RequestNotFound { request_id });
    }
    Err(e) => {
        tracing::error!(request_id = %request_id, error = %e, "repository lookup failed");
        return Err(AssignDriverError::Repository(e));
    }
}
```

Avoid logging the same failure at every layer. Let the use case or application
service own the authoritative log line and propagate the typed error upward.

## Use `tracing` Only When Helpful

`tracing` is a convenient implementation of these guidelines, but it is not a
required dependency. Use it when the project already needs structured logs,
spans, or correlation. Otherwise, apply the same principles with whatever
logging facade or custom writer the project uses.

When `tracing` is used:

- Keep spans around use-case or application-service boundaries, not every
  internal helper. A span names the operation and carries the aggregate
  identifier.
- Prefer `#[instrument]` with explicit field lists over broad auto-derived
  fields. Avoid instrumenting functions that receive raw DTOs or sensitive
  payloads unless those values are excluded.
- Use field value syntax that matches your redaction policy. `%field` records
  `Display`, `?field` records `Debug`; both must produce a safe representation
  for domain objects that contain PII.

```rust
#[tracing::instrument(
    name = "use_case.assign_driver",
    skip(driver), // skip fields that need manual redaction
    fields(request_id = %request_id, driver_id = %driver.id)
)]
pub async fn assign_driver(
    &self,
    request_id: RequestId,
    driver: DriverAssignment,
) -> Result<Transition<EnRouteRequest, TaxiRequestEvent>, AssignDriverError> {
    // ...
}
```

Do not treat `tracing` spans as domain events or audit records. They are
observability aids; persist business events through domain event types or an
outbox when durability is required.

## Measure Domain Outcomes

Metrics should reflect business outcomes, not only runtime machinery. Keep
counters, histograms, and gauges aligned with the domain concepts this skill
already models.

- **Counters**: domain events such as transitions, commands accepted/rejected,
  or published events.
- **Histograms**: durations and sizes with meaningful dimensions, such as time
  spent in each aggregate state or the latency of a use-case execution.
- **Gauges**: point-in-state values such as the number of currently waiting
  requests.

```rust
metrics::counter!("taxi_request.driver_assigned", "fleet" => fleet.as_str()).increment(1);
metrics::histogram!("taxi_request.state_duration_seconds", "from" => "waiting", "to" => "en-route")
    .record(duration.as_secs_f64());
```

Use consistent labels derived from domain types, and keep cardinality low
enough for time-series storage. Prefer a bounded set of state names or command
names over raw IDs or timestamps as label values.

## Export Telemetry Through OpenTelemetry

Use OpenTelemetry as the default application-level path for exporting logs,
metrics, and traces to observability backends. Keep domain and use-case code on
facade APIs (`tracing`, `metrics`) and wire exporters only at application
startup.

Facades do not connect to OpenTelemetry automatically. Install bridge crates at
startup:

- `tracing` spans/traces: `tracing-opentelemetry` or an equivalent project choice
- `metrics` instruments: `metrics-exporter-otel`, `metrics-opentelemetry`, or
  another recorder that forwards to an OpenTelemetry `Meter`

Pull-style `/metrics` endpoints for Prometheus scraping are optional. Prefer
OTLP export through the OpenTelemetry SDK when the deployment supports it; add a
Prometheus text exporter only when scraping is required. The legacy
`opentelemetry-prometheus` crate is deprecated; for Prometheus text exposition,
prefer `opentelemetry-prometheus-text-exporter` or route OTLP metrics through a
collector.

Do not design the domain or application layer around a specific exporter.

```rust
// Application startup, not domain code.
use opentelemetry::global;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_prometheus_text_exporter::PrometheusExporter;

// Bridge `metrics` facade recordings into OpenTelemetry.
use metrics_exporter_otel::OpenTelemetryRecorder;

let exporter = PrometheusExporter::builder().build();
let provider = SdkMeterProvider::builder()
    .with_reader(exporter)
    .build();

let meter = provider.meter(env!("CARGO_PKG_NAME"));
let recorder = OpenTelemetryRecorder::new(meter);
metrics::set_global_recorder(recorder).expect("install metrics recorder");

global::set_meter_provider(provider.clone());

// For `tracing`, install a `tracing-opentelemetry` layer separately.
```

## Correlate Logs and Metrics

Carry a correlation identifier through the request, command, or transaction.
Include it in structured logs and, when practical, expose it as a metric label
or trace attribute so operations can pivot between logs, metrics, and traces.

```rust
let correlation_id = CorrelationId::generate();
tracing::Span::current().record("correlation_id", correlation_id.as_str());
```

Keep tracing spans around use-case boundaries rather than every internal call.
The span should name the operation and carry the aggregate identifier, not the
thread of execution details.
