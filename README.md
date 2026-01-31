# Envflag

A strict, zero-boilerplate environment variable manager with `.env` support and validation.

`envflag` enforces a disciplined approach to configuration: you must explicitly initialize the library (which loads `.env` files via `dotenvy`), and then query typed values through a validated builder API or simple convenience functions.

## Features

- **Strict Initialization**: All queries panic if `init()` has not been called — no silent misconfiguration.
- **Dotenv Support**: Seamlessly loads `.env` files upon initialization, or from a custom path.
- **Prefix Filtering**: Keep only environment variables matching configured prefixes (e.g. `APP_`, `SVC_`).
- **Validated Builder API**: Chain `.default()`, `.validate()`, and `.get()` for type-safe, validated lookups that return `Result`.
- **Built-in Validators**: `is_port`, `is_integer`, `is_positive_number`, `is_bool`, `is_url`, and more.
- **Custom Validators**: Pass any `Fn(&str) -> bool` closure as a validator.
- **Zero Boilerplate**: No built-in logging or printing; you control how to display your config.

## Usage Examples

Check the `examples` directory for runnable code:

- **Basic Usage**: [`examples/basic.rs`](examples/basic.rs) - Initialize and query with convenience API.
- **Validation**: [`examples/validation.rs`](examples/validation.rs) - Chain validators on environment variables.
- **Prefix Filtering**: [`examples/prefixes.rs`](examples/prefixes.rs) - Filter and scope variables by prefix.
- **Custom Init**: [`examples/custom_init.rs`](examples/custom_init.rs) - Load from a specific `.env` file path.

## Installation

```toml
[dependencies]
envflag = { version = "0.1", features = ["full"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `tracing` | Enables optional `tracing::warn` on validation failures. |
| `regex` | Enables `matches_regex` validator via `fancy-regex`. |
| `full` | Enables all features above. |

## License

Released under the MIT License © 2026 [Canmi](https://github.com/canmi21)
