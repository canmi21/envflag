/* examples/custom_init.rs */

//! Demonstrates initializing from a specific custom .env file path.
//!
//! To run this example, you might need a dummy file at "config/.env.prod".
//! This example just shows the API usage.

use std::path::Path;

fn main() {
	// Attempt to load from a custom path.
	// If the file doesn't exist, it will return an error (unlike default init which ignores missing .env).
	let path = Path::new("config/.env.custom");

	match envflag::init_from(path) {
		Ok(_) => println!("Initialized from {:?}", path),
		Err(e) => {
			println!(
				"Could not load custom .env (this is expected if file is missing): {}",
				e
			);
			// Fallback to default init for the sake of the example running
			envflag::init().ok();
		}
	}

	println!("Envflag is ready.");
}
