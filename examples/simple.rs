/* examples/simple.rs */

use std::env;

fn main() {
	// Manually set some environment variables for the example
	// In a real scenario, these would be set by the shell or system
	env::set_var("APP_PORT", "8080");
	env::set_var("APP_DEBUG", "true");
	env::set_var("APP_NAME", "DemoApp");

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
