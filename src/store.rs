/* src/store.rs */

//! Internal environment storage and initialization.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::error::EnvflagError;

/// Global instance of the environment store.
pub(crate) static INSTANCE: OnceLock<EnvStore> = OnceLock::new();

/// Internal storage for environment variables and configuration.
///
/// This type holds the loaded environment variables and any configured
/// prefixes.  It is normally created via [`InitBuilder`] and stored in a
/// global [`OnceLock`], but can also be constructed directly with
/// [`EnvStore::from_map`] for unit-testing purposes.
#[derive(Debug)]
pub struct EnvStore {
	map: HashMap<String, String>,
	prefixes: Vec<String>,
}

impl EnvStore {
	pub(crate) fn get_instance() -> Result<&'static Self, EnvflagError> {
		INSTANCE.get().ok_or(EnvflagError::NotInitialized)
	}

	/// Creates an `EnvStore` directly from a map of key-value pairs.
	///
	/// This is intended for **testing**: it lets you construct a store without
	/// touching the global [`OnceLock`], so every test can have its own
	/// isolated instance.
	///
	/// # Examples
	///
	/// ```rust
	/// use std::collections::HashMap;
	/// use envflag::store::EnvStore;
	///
	/// let store = EnvStore::from_map(HashMap::from([
	///     ("PORT".into(), "8080".into()),
	/// ]));
	/// assert_eq!(store.lookup("PORT", None), Some("8080".to_owned()));
	/// ```
	#[must_use]
	pub fn from_map(map: HashMap<String, String>) -> Self {
		Self {
			map,
			prefixes: Vec::new(),
		}
	}

	/// Creates an `EnvStore` from a map with the given prefixes.
	///
	/// Same as [`from_map`](Self::from_map) but also sets prefix
	/// configuration, useful for testing prefix-related logic.
	#[must_use]
	pub fn from_map_with_prefixes(map: HashMap<String, String>, prefixes: Vec<String>) -> Self {
		Self { map, prefixes }
	}

	/// Looks up a key in the store.
	///
	/// When prefixes are configured:
	/// - Single prefix: automatically prepends it to reconstruct the original key.
	/// - Multiple prefixes: `preferred_prefix` **must** be specified; otherwise returns `None`.
	///
	/// When no prefixes are configured, looks up the key directly.
	#[must_use]
	pub fn lookup(&self, key: &str, preferred_prefix: Option<&str>) -> Option<String> {
		if self.prefixes.is_empty() {
			// No prefix mode — direct lookup.
			return self.map.get(key).cloned();
		}

		// Prefix mode — reconstruct the original key.
		if let Some(p) = preferred_prefix {
			return self.map.get(&format!("{p}{key}")).cloned();
		}

		if self.prefixes.len() == 1 {
			let p = &self.prefixes[0];
			return self.map.get(&format!("{p}{key}")).cloned();
		}

		// Multiple prefixes without explicit choice — cannot resolve.
		None
	}

	/// Returns the configured prefixes.
	#[must_use]
	pub fn prefixes(&self) -> &[String] {
		&self.prefixes
	}

	/// Returns all environment variables in the store.
	#[must_use]
	pub fn entries(&self) -> Vec<(String, String)> {
		self
			.map
			.iter()
			.map(|(k, v)| (k.clone(), v.clone()))
			.collect()
	}
}

/// Builder for initializing the envflag crate.
///
/// # Initialization order
///
/// It is recommended to call `init()` early in `main()` **before** spawning
/// any threads. The global store is backed by [`OnceLock`] so concurrent
/// `init()` calls are memory-safe, but deterministic single-threaded
/// initialization avoids surprises.
#[derive(Debug)]
pub struct InitBuilder {
	path: Option<PathBuf>,
	prefixes: Vec<String>,
}

impl Default for InitBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl InitBuilder {
	/// Creates a new `InitBuilder`.
	#[must_use]
	pub fn new() -> Self {
		Self {
			path: None,
			prefixes: Vec::new(),
		}
	}

	/// Sets the path to the `.env` file.
	#[must_use]
	pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
		self.path = Some(path.as_ref().to_path_buf());
		self
	}

	/// Adds a prefix to filter environment variables.
	///
	/// Only keys matching at least one configured prefix will be kept in the
	/// internal store (stored under their **original** full key).  At query
	/// time the prefix is automatically prepended so callers use the short
	/// name (e.g. `get("PORT", 8080)` finds `APP_PORT`).
	#[must_use]
	pub fn prefix(mut self, prefix: &str) -> Self {
		self.prefixes.push(prefix.to_owned());
		self
	}

	/// Initializes the global environment store.
	///
	/// # Errors
	///
	/// Returns an error if the crate is already initialized, or if the `.env`
	/// file cannot be loaded.
	pub fn init(self) -> Result<(), EnvflagError> {
		// 1. Load dotenv into std::env
		if let Some(p) = self.path {
			dotenvy::from_path(p)?;
		} else {
			match dotenvy::dotenv() {
				Ok(_) => {}
				Err(e) if e.not_found() => {}
				Err(e) => return Err(EnvflagError::Dotenv(e)),
			}
		}

		// 2. Collect env vars into private map
		let all_vars: HashMap<String, String> = env::vars().collect();
		let map = if self.prefixes.is_empty() {
			all_vars
		} else {
			// Strict filter: only keep keys that match a configured prefix.
			all_vars
				.into_iter()
				.filter(|(k, _)| self.prefixes.iter().any(|p| k.starts_with(p)))
				.collect()
		};

		let store = EnvStore {
			map,
			prefixes: self.prefixes,
		};

		// OnceLock::set is atomic — no TOCTOU possible.
		INSTANCE
			.set(store)
			.map_err(|_| EnvflagError::AlreadyInitialized)?;
		Ok(())
	}
}
