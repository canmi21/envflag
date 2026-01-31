/* examples/validation.rs */

//! Demonstrates the Chain API with validation and error handling.
//!
//! Run with:
//! PORT=9090 DATABASE_URL=postgres://localhost/db cargo run --example validation

use envflag::validators;

fn main() -> Result<(), envflag::EnvflagError> {
	envflag::init()?;

	println!(
		"Run with: PORT=9090 DATABASE_URL=postgres://localhost/db cargo run --example validation\n"
	);

	// 1. Validated Optional Value
	// Ensures PORT is a valid u16 and a valid port number.
	let port = envflag::key("PORT")
		.default(8080)
		.validate(validators::is_port)
		.get()?;

	println!("Port: {}", port);

	// 2. Required Value
	// Currently, .required() is a terminal method.
	let db_url: String = envflag::key("DATABASE_URL").required()?;

	// If manual validation is needed for required values:
	if !validators::is_url(&db_url) {
		eprintln!("Warning: DATABASE_URL does not look like a valid URL");
	}

	println!("Database URL: {}", db_url);

	Ok(())
}
