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
#[must_use]
pub fn is_bool(s: &str) -> bool {
	let s = s.trim().to_lowercase();
	matches!(s.as_str(), "true" | "1" | "yes" | "false" | "0" | "no")
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
