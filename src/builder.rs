/* src/builder.rs */

//! Chained query builder for environment variables.

use crate::error::EnvflagError;
use crate::store::EnvStore;
use std::any::TypeId;
use std::fmt;
use std::str::FromStr;

/// Builder for querying a specific environment variable.
#[derive(Debug)]
pub struct KeyBuilder<'a> {
	name: &'a str,
	prefix: Option<&'a str>,
}

impl<'a> KeyBuilder<'a> {
	pub(crate) fn new(name: &'a str) -> Self {
		Self { name, prefix: None }
	}

	/// Specifies which prefix to use for this lookup.
	///
	/// Required when multiple prefixes are configured; optional with a single
	/// prefix (which is used automatically).
	#[must_use]
	pub fn with_prefix(mut self, prefix: &'a str) -> Self {
		self.prefix = Some(prefix);
		self
	}

	/// Sets a default value and transitions to a typed builder.
	#[must_use]
	pub fn default<T: ToString>(self, val: T) -> TypedKeyBuilder<'a, T> {
		TypedKeyBuilder {
			name: self.name,
			prefix: self.prefix,
			default_val: val,
			validators: Vec::new(),
		}
	}

	/// Marks the variable as required.
	///
	/// # Errors
	///
	/// Returns `EnvflagError::NotSet` if the variable is missing,
	/// `EnvflagError::ParseFailed` if parsing fails, or
	/// `EnvflagError::AmbiguousPrefix` if multiple prefixes are configured
	/// without an explicit `with_prefix` call.
	pub fn required<T: FromStr + 'static>(self) -> Result<T, EnvflagError> {
		let store = EnvStore::get_instance()?;

		if store.prefixes().len() > 1 && self.prefix.is_none() {
			return Err(EnvflagError::AmbiguousPrefix {
				key: self.name.to_owned(),
			});
		}

		let raw = store
			.lookup(self.name, self.prefix)
			.ok_or_else(|| EnvflagError::NotSet {
				key: self.name.to_owned(),
			})?;

		let val_str = if TypeId::of::<T>() == TypeId::of::<bool>() {
			crate::validators::normalize_bool(&raw).into_owned()
		} else {
			raw
		};

		val_str.parse::<T>().map_err(|_| EnvflagError::ParseFailed {
			key: self.name.to_owned(),
			value: val_str,
		})
	}
}

/// A builder for a specific key with a default value and optional validators.
pub struct TypedKeyBuilder<'a, T> {
	name: &'a str,
	prefix: Option<&'a str>,
	default_val: T,
	validators: Vec<Box<dyn Fn(&str) -> bool>>,
}

impl<T: fmt::Debug> fmt::Debug for TypedKeyBuilder<'_, T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("TypedKeyBuilder")
			.field("name", &self.name)
			.field("prefix", &self.prefix)
			.field("default_val", &self.default_val)
			.field(
				"validators",
				&format!("[{} validator(s)]", self.validators.len()),
			)
			.finish()
	}
}

impl<'a, T> TypedKeyBuilder<'a, T>
where
	T: FromStr + ToString + 'static,
{
	/// Adds a validator function to be run against the raw string value.
	///
	/// Multiple validators can be chained; all must pass.
	/// Accepts any `Fn(&str) -> bool`, including closures and function pointers.
	#[must_use]
	pub fn validate(mut self, f: impl Fn(&str) -> bool + 'static) -> Self {
		self.validators.push(Box::new(f));
		self
	}

	/// Executes the query and returns the parsed value or the default.
	///
	/// # Errors
	///
	/// - `EnvflagError::ValidationFailed` if any validator fails.
	/// - `EnvflagError::ParseFailed` if parsing fails.
	/// - `EnvflagError::AmbiguousPrefix` if multiple prefixes are configured
	///   without an explicit `with_prefix` call.
	pub fn get(self) -> Result<T, EnvflagError> {
		let store = EnvStore::get_instance()?;

		if store.prefixes().len() > 1 && self.prefix.is_none() {
			return Err(EnvflagError::AmbiguousPrefix {
				key: self.name.to_owned(),
			});
		}

		let val_str_opt = store.lookup(self.name, self.prefix);

		match val_str_opt {
			Some(raw) => {
				// Normalize booleans before validation so validators see
				// the canonical "true"/"false" form.
				let val_str = if TypeId::of::<T>() == TypeId::of::<bool>() {
					crate::validators::normalize_bool(&raw).into_owned()
				} else {
					raw
				};

				// Run validators
				for v in &self.validators {
					if !v(&val_str) {
						#[cfg(feature = "tracing")]
						tracing::warn!(
							key = %self.name,
							value = %val_str,
							"validation failed for environment variable"
						);
						return Err(EnvflagError::ValidationFailed {
							key: self.name.to_owned(),
							value: val_str,
						});
					}
				}

				// Parse
				val_str.parse::<T>().map_err(|_| EnvflagError::ParseFailed {
					key: self.name.to_owned(),
					value: val_str,
				})
			}
			None => Ok(self.default_val),
		}
	}
}
