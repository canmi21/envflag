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
//!
//! ## Testing
//!
//! For unit tests, construct an [`EnvStore`](store::EnvStore) directly via
//! [`EnvStore::from_map`](store::EnvStore::from_map) and call its instance
//! methods ([`get`](store::EnvStore::get), [`key`](store::EnvStore::key),
//! etc.) instead of the global functions. This avoids the `OnceLock` and
//! gives each test its own isolated store.

/// Chained query builder for environment variables.
pub mod builder;
/// Error types for the crate.
pub mod error;
/// Internal environment storage and initialization.
pub mod store;
/// Built-in validation functions.
pub mod validators;

use std::any::TypeId;
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
/// Unlike [`init()`] which silently ignores a missing `.env` file, this
/// function **requires** the file to exist and will return an error if it
/// cannot be loaded.
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
pub fn get<T: FromStr + 'static>(name: &str, default: T) -> T {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.get(name, default)
}

/// Retrieves an environment variable as a String.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn get_string(name: &str, default: &str) -> String {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.get_string(name, default)
}

/// Retrieves an environment variable and parses it, returning `None` if not set or parse fails.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn lookup<T: FromStr + 'static>(name: &str) -> Option<T> {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.lookup_parsed(name)
}

/// Retrieves an environment variable as a String, returning `None` if not set.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn lookup_string(name: &str) -> Option<String> {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.lookup_string(name)
}

/// Checks if an environment variable is set.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn is_set(name: &str) -> bool {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.is_set(name)
}

/// Returns all environment variables in the store.
///
/// **Warning:** This returns all stored entries including potentially sensitive
/// values (e.g. `DATABASE_URL`, `SECRET_KEY`). If you need to expose these
/// (e.g. for debugging), consider filtering or redacting secrets yourself.
///
/// # Panics
///
/// Panics if the crate has not been initialized.
#[must_use]
pub fn entries() -> Vec<(String, String)> {
	let store = store::EnvStore::get_instance().expect("envflag is not initialized");
	store.entries()
}

// ---------------------------------------------------------------------------
// Instance methods on EnvStore — the real logic lives here.
// ---------------------------------------------------------------------------

impl store::EnvStore {
	/// Starts a chained query against this store instance.
	///
	/// Works exactly like the global [`key()`] function but without requiring
	/// global initialization, making it ideal for tests.
	///
	/// # Examples
	///
	/// ```rust
	/// use std::collections::HashMap;
	/// use envflag::store::EnvStore;
	///
	/// let store = EnvStore::from_map(HashMap::from([
	///     ("PORT".into(), "3000".into()),
	/// ]));
	/// let port: u16 = store.key("PORT").default(8080u16).get().unwrap();
	/// assert_eq!(port, 3000);
	/// ```
	#[must_use]
	pub fn key<'a>(&'a self, name: &'a str) -> KeyBuilder<'a> {
		KeyBuilder::new_with_store(name, self)
	}

	/// Retrieves an environment variable and parses it into the specified type.
	///
	/// If the variable is missing or cannot be parsed, returns `default`.
	pub fn get<T: FromStr + 'static>(&self, name: &str, default: T) -> T {
		match self.lookup(name, None) {
			Some(val) => {
				let val = if TypeId::of::<T>() == TypeId::of::<bool>() {
					crate::validators::normalize_bool(&val)
				} else {
					std::borrow::Cow::Borrowed(val.as_str())
				};
				if let Ok(v) = val.parse::<T>() {
					v
				} else {
					#[cfg(feature = "tracing")]
					tracing::warn!(
						key = %name,
						value = %val,
						"failed to parse environment variable, using default"
					);
					default
				}
			}
			None => default,
		}
	}

	/// Retrieves an environment variable as a `String`.
	///
	/// If not set, returns `default`.
	#[must_use]
	pub fn get_string(&self, name: &str, default: &str) -> String {
		self
			.lookup(name, None)
			.unwrap_or_else(|| default.to_owned())
	}

	/// Retrieves an environment variable and parses it, returning `None` if
	/// not set or if parsing fails.
	#[must_use]
	pub fn lookup_parsed<T: FromStr + 'static>(&self, name: &str) -> Option<T> {
		self.lookup(name, None).and_then(|s| {
			let s = if TypeId::of::<T>() == TypeId::of::<bool>() {
				crate::validators::normalize_bool(&s)
			} else {
				std::borrow::Cow::Borrowed(s.as_str())
			};
			#[allow(clippy::manual_ok_err)]
			if let Ok(v) = s.parse::<T>() {
				Some(v)
			} else {
				#[cfg(feature = "tracing")]
				tracing::warn!(
					key = %name,
					value = %s,
					"failed to parse environment variable, returning None"
				);
				None
			}
		})
	}

	/// Retrieves an environment variable as a `String`, returning `None` if
	/// not set.
	#[must_use]
	pub fn lookup_string(&self, name: &str) -> Option<String> {
		self.lookup(name, None)
	}

	/// Checks if an environment variable is set in this store.
	#[must_use]
	pub fn is_set(&self, name: &str) -> bool {
		self.lookup(name, None).is_some()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashMap;
	use store::EnvStore;

	// This test must run in a separate process because OnceLock cannot be
	// reset.  `cargo test` runs each test binary once; as long as no other
	// test in *this* binary calls init() before this test, it works.
	#[test]
	#[should_panic(expected = "envflag is not initialized")]
	fn test_panic_uninitialized() {
		// Intentionally do NOT call init().
		let _ = is_set("ANY");
	}

	// ---- EnvStore instance tests (no OnceLock needed) --------------------

	fn make_store(pairs: &[(&str, &str)]) -> EnvStore {
		EnvStore::from_map(
			pairs
				.iter()
				.map(|(k, v)| ((*k).into(), (*v).into()))
				.collect(),
		)
	}

	#[test]
	fn get_existing_key() {
		let store = make_store(&[("PORT", "3000")]);
		assert_eq!(store.get::<u16>("PORT", 8080), 3000);
	}

	#[test]
	fn get_missing_key_returns_default() {
		let store = make_store(&[]);
		assert_eq!(store.get::<u16>("PORT", 8080), 8080);
	}

	#[test]
	fn get_unparseable_returns_default() {
		let store = make_store(&[("PORT", "abc")]);
		assert_eq!(store.get::<u16>("PORT", 8080), 8080);
	}

	#[test]
	fn get_bool_normalizes() {
		let store = make_store(&[("DEBUG", "yes"), ("VERBOSE", "0")]);
		assert!(store.get::<bool>("DEBUG", false));
		assert!(!store.get::<bool>("VERBOSE", true));
	}

	#[test]
	fn get_string_existing() {
		let store = make_store(&[("HOST", "localhost")]);
		assert_eq!(store.get_string("HOST", "0.0.0.0"), "localhost");
	}

	#[test]
	fn get_string_missing() {
		let store = make_store(&[]);
		assert_eq!(store.get_string("HOST", "0.0.0.0"), "0.0.0.0");
	}

	#[test]
	fn lookup_parsed_existing() {
		let store = make_store(&[("PORT", "9090")]);
		assert_eq!(store.lookup_parsed::<u16>("PORT"), Some(9090));
	}

	#[test]
	fn lookup_parsed_missing() {
		let store = make_store(&[]);
		assert_eq!(store.lookup_parsed::<u16>("PORT"), None);
	}

	#[test]
	fn lookup_parsed_unparseable() {
		let store = make_store(&[("PORT", "xyz")]);
		assert_eq!(store.lookup_parsed::<u16>("PORT"), None);
	}

	#[test]
	fn lookup_string_existing() {
		let store = make_store(&[("HOST", "localhost")]);
		assert_eq!(store.lookup_string("HOST"), Some("localhost".to_owned()));
	}

	#[test]
	fn lookup_string_missing() {
		let store = make_store(&[]);
		assert_eq!(store.lookup_string("HOST"), None);
	}

	#[test]
	fn is_set_true() {
		let store = make_store(&[("A", "1")]);
		assert!(store.is_set("A"));
	}

	#[test]
	fn is_set_false() {
		let store = make_store(&[]);
		assert!(!store.is_set("A"));
	}

	#[test]
	fn entries_returns_all() {
		let store = make_store(&[("A", "1"), ("B", "2")]);
		let mut e = store.entries();
		e.sort();
		assert_eq!(
			e,
			vec![
				("A".to_owned(), "1".to_owned()),
				("B".to_owned(), "2".to_owned()),
			]
		);
	}

	// ---- Builder API via store.key() ------------------------------------

	#[test]
	fn key_default_existing() {
		let store = make_store(&[("PORT", "3000")]);
		let v: u16 = store.key("PORT").default(8080u16).get().unwrap();
		assert_eq!(v, 3000);
	}

	#[test]
	fn key_default_missing() {
		let store = make_store(&[]);
		let v: u16 = store.key("PORT").default(8080u16).get().unwrap();
		assert_eq!(v, 8080);
	}

	#[test]
	fn key_required_existing() {
		let store = make_store(&[("PORT", "3000")]);
		let v: u16 = store.key("PORT").required().unwrap();
		assert_eq!(v, 3000);
	}

	#[test]
	fn key_required_missing() {
		let store = make_store(&[]);
		let err = store.key("PORT").required::<u16>().unwrap_err();
		assert!(matches!(err, EnvflagError::NotSet { .. }));
	}

	#[test]
	fn key_validate_pass() {
		let store = make_store(&[("PORT", "8080")]);
		let v: u16 = store
			.key("PORT")
			.default(3000u16)
			.validate(validators::is_port)
			.get()
			.unwrap();
		assert_eq!(v, 8080);
	}

	#[test]
	fn key_validate_fail() {
		let store = make_store(&[("PORT", "0")]);
		let err = store
			.key("PORT")
			.default(3000u16)
			.validate(validators::is_port)
			.get()
			.unwrap_err();
		assert!(matches!(err, EnvflagError::ValidationFailed { .. }));
	}

	#[test]
	fn key_parse_fail() {
		let store = make_store(&[("PORT", "abc")]);
		let err = store.key("PORT").required::<u16>().unwrap_err();
		assert!(matches!(err, EnvflagError::ParseFailed { .. }));
	}

	#[test]
	fn key_bool_normalization() {
		let store = make_store(&[("FLAG", "yes")]);
		let v: bool = store.key("FLAG").required().unwrap();
		assert!(v);
	}

	// ---- Prefix tests ---------------------------------------------------

	#[test]
	fn single_prefix_auto() {
		let store = EnvStore::from_map_with_prefixes(
			HashMap::from([("APP_PORT".into(), "3000".into())]),
			vec!["APP_".into()],
		);
		let v: u16 = store.key("PORT").default(8080u16).get().unwrap();
		assert_eq!(v, 3000);
	}

	#[test]
	fn multi_prefix_explicit() {
		let store = EnvStore::from_map_with_prefixes(
			HashMap::from([
				("APP_PORT".into(), "3000".into()),
				("SVC_PORT".into(), "4000".into()),
			]),
			vec!["APP_".into(), "SVC_".into()],
		);
		let v: u16 = store
			.key("PORT")
			.with_prefix("SVC_")
			.default(8080u16)
			.get()
			.unwrap();
		assert_eq!(v, 4000);
	}

	#[test]
	fn multi_prefix_ambiguous() {
		let store = EnvStore::from_map_with_prefixes(
			HashMap::from([("APP_PORT".into(), "3000".into())]),
			vec!["APP_".into(), "SVC_".into()],
		);
		let err = store.key("PORT").default(8080u16).get().unwrap_err();
		assert!(matches!(err, EnvflagError::AmbiguousPrefix { .. }));
	}
}
