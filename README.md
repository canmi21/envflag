# Envflag

A lightweight utility to read and parse environment variables into any type.

`envflag` provides a simple, one-line API to fetch environment variables and automatically parse them into the desired type, with fallback to default values. It is designed to be zero-dependency (when using `std`) and easy to integrate into any Rust project.

## Features

- **Generic Parsing**: Automatically parse environment variables into any type that implements `FromStr`.
- **String Support**: Specialized helper for reading environment variables as `String`.
- **Boolean Parsing**: Robust boolean parsing supporting "true", "1", and "yes".
- **Zero Dependencies**: Pure Rust implementation with no external dependencies.
- **Safe Fallbacks**: Always returns a default value if the environment variable is missing or invalid.

## Usage Examples

Check the `examples` directory for runnable code:

- **Basic Usage**: [`examples/simple.rs`](examples/simple.rs) - Demonstrate reading various types of environment variables.

## Installation

```toml
[dependencies]
envflag = { version = "0.0", features = ["full"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `std` | Enables standard library support for reading environment variables. |
| `full` | Enables all features above. |

## License

Released under the MIT License © 2026 [Canmi](https://github.com/canmi21)