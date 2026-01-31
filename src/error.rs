/* src/error.rs */

//! Error types for the crate.

use thiserror::Error;

/// Errors that can occur when using the envflag crate.
#[derive(Debug, Error)]
pub enum EnvflagError {
	/// The crate has already been initialized.
	#[error("envflag is already initialized")]
	AlreadyInitialized,

	/// The crate has not been initialized.
	#[error(
		"envflag is not initialized! You must call envflag::init() or use the builder before querying."
	)]
	NotInitialized,

	/// An I/O error occurred.
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),

	/// A dotenv error occurred.
	#[error("dotenv error: {0}")]
	Dotenv(#[from] dotenvy::Error),

	/// The requested environment variable is not set.
	#[error("environment variable '{key}' is not set")]
	NotSet {
		/// The key that was not found.
		key: String,
	},

	/// Multiple prefixes are configured but no explicit prefix was specified.
	#[error(
		"ambiguous prefix for key '{key}': multiple prefixes configured, use .with_prefix() to specify one"
	)]
	AmbiguousPrefix {
		/// The key that could not be resolved.
		key: String,
	},

	/// Validation failed for the environment variable.
	#[error("validation failed for key '{key}' with value '{value}'")]
	ValidationFailed {
		/// The key that failed validation.
		key: String,
		/// The value that failed validation.
		value: String,
	},

	/// Parsing failed for the environment variable.
	#[error("failed to parse key '{key}' with value '{value}'")]
	ParseFailed {
		/// The key that failed parsing.
		key: String,
		/// The value that failed parsing.
		value: String,
	},
}
