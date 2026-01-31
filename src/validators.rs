/* src/validators.rs */

//! Built-in validation functions for environment variables.

/// Checks if a string is not empty or just whitespace.
#[must_use]
pub fn is_non_empty(s: &str) -> bool {
	!s.trim().is_empty()
}

/// Checks if a string can be parsed as an integer.
#[must_use]
pub fn is_integer(s: &str) -> bool {
	s.trim().parse::<i64>().is_ok()
}

/// Checks if a string can be parsed as a positive integer (> 0).
///
/// **Note:** This validates against `u64`. If you are parsing into a smaller
/// type (e.g. `u8`, `i32`), validation may pass while parsing still fails due
/// to overflow.
#[must_use]
pub fn is_positive_integer(s: &str) -> bool {
	s.trim().parse::<u64>().is_ok_and(|v| v > 0)
}

/// Checks if a string can be parsed as a positive number (> 0.0).
#[must_use]
pub fn is_positive_number(s: &str) -> bool {
	s.trim().parse::<f64>().is_ok_and(|v| v > 0.0)
}

/// Checks if a string is a valid boolean representation ("true", "1", "yes", "false", "0", "no").
///
/// This pairs with the special boolean handling in `key().get()` and `key().required()`
/// which automatically normalizes these values to "true"/"false" before parsing.
#[must_use]
pub fn is_bool(s: &str) -> bool {
	let s = s.trim().to_lowercase();
	matches!(s.as_str(), "true" | "1" | "yes" | "false" | "0" | "no")
}

/// Internal helper to normalize boolean strings.
pub(crate) fn normalize_bool(s: &str) -> std::borrow::Cow<'_, str> {
	let trimmed = s.trim();
	if trimmed.eq_ignore_ascii_case("true") {
		return std::borrow::Cow::Owned("true".to_owned());
	}
	if trimmed.eq_ignore_ascii_case("false") {
		return std::borrow::Cow::Owned("false".to_owned());
	}

	match trimmed.to_lowercase().as_str() {
		"1" | "yes" => std::borrow::Cow::Owned("true".to_owned()),
		"0" | "no" => std::borrow::Cow::Owned("false".to_owned()),
		_ => std::borrow::Cow::Borrowed(s),
	}
}

/// Returns a validator that checks if a string can be parsed as an integer
/// within the given inclusive range.
///
/// This allows type-safe range validation that matches the actual target type,
/// avoiding the mismatch where `is_positive_integer` (which uses `u64`) may
/// accept values that overflow smaller types like `u16` or `i32`.
///
/// # Examples
///
/// ```rust
/// use envflag::validators::is_integer_in_range;
///
/// let valid_u16 = is_integer_in_range(1_i64, 65535);
/// assert!(valid_u16("8080"));
/// assert!(!valid_u16("70000"));
/// assert!(!valid_u16("0"));
/// ```
pub fn is_integer_in_range(min: i64, max: i64) -> impl Fn(&str) -> bool {
	move |s| s.trim().parse::<i64>().is_ok_and(|v| v >= min && v <= max)
}

/// Checks if a string is a valid port number (1-65535).
#[must_use]
pub fn is_port(s: &str) -> bool {
	s.trim().parse::<u16>().is_ok_and(|v| v > 0)
}

/// Simple check if a string looks like a URL (contains "://").
#[must_use]
pub fn is_url(s: &str) -> bool {
	s.contains("://")
}

/// Returns a validator that checks if a string matches a regex pattern.
///
/// # Panics
///
/// Panics immediately if `pattern` is not a valid regex. This is intentional:
/// an invalid pattern is a programming error and should be caught at startup,
/// not silently ignored at query time.
#[cfg(feature = "regex")]
pub fn matches_regex(pattern: &str) -> impl Fn(&str) -> bool {
	let re = fancy_regex::Regex::new(pattern)
		.unwrap_or_else(|e| panic!("invalid regex pattern \"{pattern}\": {e}"));
	move |s| re.is_match(s).unwrap_or(false)
}
