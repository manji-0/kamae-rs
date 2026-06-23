# anyhow / eyre

Use `anyhow` or `eyre` at application edges: command handlers, main functions, migration tools, and glue code.

Do not use `anyhow::Result<T>` as the return type of domain entities, value-object constructors, or use cases that callers must handle exhaustively. Convert domain-specific errors into `anyhow` only at the reporting boundary.
