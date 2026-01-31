/* src/lib.rs */

//! # Envflag
//!
//! A strict, zero-boilerplate environment variable manager with `.env` support and validation.
//!
//! ## Initialization
//!
//! Call [`init()`] (or use the [`builder()`]) early in `main()`, **before**
//! spawning threads. All query functions will panic if the crate has not been
//! initialized.

/// Chained query builder for environment variables.
pub mod builder;
/// Error types for the crate.
pub mod error;
/// Internal environment storage and initialization.
pub mod store;
/// Built-in validation functions.
pub mod validators;

use std::path::Path;
use std::str::FromStr;

pub use builder::{KeyBuilder, TypedKeyBuilder};
pub use error::EnvflagError;
pub use store::InitBuilder;

/// Initializes the environment loader using the default `.env` file and system env.
///
/// # Errors
///
/// Returns an error if the crate is already initialized or if `.env` parsing fails.
///
/// # Examples
///
/// ```rust
/// # use envflag::EnvflagError;
/// # fn main() -> Result<(), EnvflagError> {
/// envflag::init()?;
/// # Ok(())
/// # }
/// ```
pub fn init() -> Result<(), EnvflagError> {
	InitBuilder::new().init()
}

/// Initializes the environment loader from a specific file path.
///
/// # Errors
///
/// Returns an error if the crate is already initialized or if the file cannot be loaded.
pub fn init_from<P: AsRef<Path>>(path: P) -> Result<(), EnvflagError> {
	InitBuilder::new().path(path).init()
}

/// Returns a builder for advanced initialization (prefixes, custom paths).
#[must_use]
pub fn builder() -> InitBuilder {
	InitBuilder::new()
}

/// Starts a chained query for an environment variable.
///
/// This is the **recommended** API. Use it with `.default()` / `.required()`
/// and optional `.validate()` calls.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
///
/// # Examples
///
/// ```rust
/// # envflag::init().ok();
/// let port = envflag::key("PORT").default(8080u16).get();
/// ```
#[must_use]
pub fn key(name: &str) -> KeyBuilder<'_> {
	KeyBuilder::new(name)
}

// --- Convenience API ---
// These are simple wrappers that silently fall back to the default on any
// error.  For production config it is recommended to use the `key()` builder
// which returns `Result` and supports validation.

/// Retrieves an environment variable and parses it into the specified type.
///
/// If the variable is missing or cannot be parsed, returns the `default` value.
///
/// **Note:** When multiple prefixes are configured this function cannot
/// resolve the key and will return `default`. Use [`key()`] with
/// `.with_prefix()` instead.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
pub fn get<T: FromStr>(name: &str, default: T) -> T {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	match store.lookup(name, None) {
		Some(val) => val.parse::<T>().unwrap_or(default),
		None => default,
	}
}

/// Retrieves an environment variable as a String.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn get_string(name: &str, default: &str) -> String {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store
		.lookup(name, None)
		.unwrap_or_else(|| default.to_owned())
}

/// Retrieves an environment variable and parses it, returning `None` if not set or parse fails.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn lookup<T: FromStr>(name: &str) -> Option<T> {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.lookup(name, None).and_then(|s| s.parse::<T>().ok())
}

/// Retrieves an environment variable as a String, returning `None` if not set.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn lookup_string(name: &str) -> Option<String> {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.lookup(name, None)
}

/// Checks if an environment variable is set.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn is_set(name: &str) -> bool {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.lookup(name, None).is_some()
}

/// Returns all environment variables in the store.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn entries() -> Vec<(String, String)> {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.entries()
}

#[cfg(test)]
mod tests {
	use super::*;

	// This test must run in a separate process because OnceLock cannot be
	// reset.  `cargo test` runs each test binary once; as long as no other
	// test in *this* binary calls init() before this test, it works.
	#[test]
	#[should_panic(expected = "envflag is not initialized")]
	fn test_panic_uninitialized() {
		// Intentionally do NOT call init().
		let _ = is_set("ANY");
	}
}
