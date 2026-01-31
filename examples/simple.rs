/* examples/simple.rs */

//! Example of using envflag to read environment variables.

fn main() {
	println!("To test with custom values, run:");
	println!("  APP_PORT=8080 APP_DEBUG=true APP_NAME=MyApp cargo run --example simple\n");

	// Read integer with default
	let port: u16 = envflag::get("APP_PORT", 3000);
	println!("Port: {}", port);

	// Read boolean with default
	// Supports "true", "1", "yes" as true
	let debug_mode = envflag::get_bool("APP_DEBUG", false);
	println!("Debug Mode: {}", debug_mode);

	// Read string with default
	let app_name = envflag::get_string("APP_NAME", "Unknown");
	println!("App Name: {}", app_name);

	// Demonstrate fallback
	let database_url = envflag::get_string("DATABASE_URL", "postgres://localhost:5432/db");
	println!("Database URL: {}", database_url);
}
