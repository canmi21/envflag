/* src/lib.rs */

#[cfg(feature = "std")]
use std::env;
#[cfg(feature = "std")]
use std::str::FromStr;

/// Retrieves an environment variable and parses it into the specified type.
///
/// If the environment variable is not present or cannot be parsed, the `default` value is returned.
///
/// # Examples
///
/// ```rust
/// // Assuming ENV_VAR is set to "42"
/// // std::env::set_var("ENV_VAR", "42");
/// let val: i32 = envflag::get("ENV_VAR", 0);
/// // val is 42 (if variable is set)
/// ```
#[cfg(feature = "std")]
pub fn get<T: FromStr>(key: &str, default: T) -> T {
	match env::var(key) {
		Ok(val) => val.parse::<T>().unwrap_or(default),
		Err(_) => default,
	}
}

/// Retrieves an environment variable as a String.
///
/// If the environment variable is not present, the `default` value is returned as a String.
///
/// # Examples
///
/// ```rust
/// let val = envflag::get_string("USER", "unknown");
/// ```
#[cfg(feature = "std")]
pub fn get_string(key: &str, default: &str) -> String {
	env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Retrieves an environment variable and parses it as a boolean.
///
/// The following case-insensitive values are considered `true`:
/// - "true"
/// - "1"
/// - "yes"
///
/// All other values (including empty strings) are considered `false`.
/// If the environment variable is not set, the `default` value is returned.
///
/// # Examples
///
/// ```rust
/// let flag = envflag::get_bool("ENABLE_FEATURE", false);
/// ```
#[cfg(feature = "std")]
pub fn get_bool(key: &str, default: bool) -> bool {
	match env::var(key) {
		Ok(val) => {
			let s = val.trim().to_lowercase();
			matches!(s.as_str(), "true" | "1" | "yes")
		}
		Err(_) => default,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use temp_env;

	#[test]
	fn test_get_int() {
		temp_env::with_var("TEST_INT", Some("123"), || {
			assert_eq!(get("TEST_INT", 0), 123);
		});
	}

	#[test]
	fn test_get_int_default() {
		temp_env::with_var("TEST_INT_MISSING", None::<&str>, || {
			assert_eq!(get("TEST_INT_MISSING", 42), 42);
		});
	}

	#[test]
	fn test_get_int_invalid() {
		temp_env::with_var("TEST_INT_INVALID", Some("abc"), || {
			assert_eq!(get("TEST_INT_INVALID", 42), 42);
		});
	}

	#[test]
	fn test_get_string() {
		temp_env::with_var("TEST_STR", Some("hello"), || {
			assert_eq!(get_string("TEST_STR", "world"), "hello");
		});
	}

	#[test]
	fn test_get_string_default() {
		temp_env::with_var("TEST_STR_MISSING", None::<&str>, || {
			assert_eq!(get_string("TEST_STR_MISSING", "world"), "world");
		});
	}

	#[test]
	fn test_get_bool_true_variants() {
		let true_vals = ["true", "True", "TRUE", "1", "yes", "Yes", "YES"];
		for val in true_vals {
			temp_env::with_var("TEST_BOOL", Some(val), || {
				assert!(get_bool("TEST_BOOL", false), "Failed for value: {}", val);
			});
		}
	}

	#[test]
	fn test_get_bool_false_variants() {
		let false_vals = ["false", "0", "no", "foo", ""];
		for val in false_vals {
			temp_env::with_var("TEST_BOOL", Some(val), || {
				assert!(!get_bool("TEST_BOOL", true), "Failed for value: {}", val);
			});
		}
	}

	#[test]
	fn test_get_bool_default() {
		temp_env::with_var("TEST_BOOL_MISSING", None::<&str>, || {
			assert!(get_bool("TEST_BOOL_MISSING", true));
			assert!(!get_bool("TEST_BOOL_MISSING", false));
		});
	}
}
