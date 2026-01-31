/* examples/basic.rs */

//! Demonstrates the basic usage of envflag: initialization and convenience API.
//!
//! Run with:
//! PORT=9090 HOST=127.0.0.1 cargo run --example basic

fn main() {
	// 1. Initialize (Required)
	// Loads .env if present, otherwise uses system environment.
	envflag::init().expect("Failed to initialize envflag");

	println!("Run with: PORT=9090 HOST=127.0.0.1 cargo run --example basic\n");

	// 2. Convenience API
	// Use simple get/get_string for common use cases.
	let port: u16 = envflag::get("PORT", 8080);
	let host = envflag::get_string("HOST", "localhost");
	let debug = envflag::get("DEBUG", false);

	println!("Host: {}", host);
	println!("Port: {}", port);
	println!("Debug: {}", debug);
}
