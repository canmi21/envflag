/* examples/prefixes.rs */

//! Demonstrates prefix filtering and lookup.
//!
//! Run with:
//! APP_PORT=3000 APP_HOST=0.0.0.0 SVC_PORT=4000 cargo run --example prefixes

fn main() {
	// Configure multiple prefixes.
	// Only keys starting with APP_ or SVC_ are kept in the store.
	envflag::builder()
		.prefix("APP_")
		.prefix("SVC_")
		.init()
		.expect("Failed to init");

	println!("Run with: APP_PORT=3000 APP_HOST=0.0.0.0 SVC_PORT=4000 cargo run --example prefixes\n");

	// With multiple prefixes you must specify which one to use:
	let app_port: u16 = envflag::key("PORT")
		.with_prefix("APP_")
		.default(8080u16)
		.get()
		.unwrap_or(8080);

	let svc_port: u16 = envflag::key("PORT")
		.with_prefix("SVC_")
		.default(9090u16)
		.get()
		.unwrap_or(9090);

	println!("APP_PORT: {app_port}");
	println!("SVC_PORT: {svc_port}");

	// Without with_prefix() on multiple prefixes, you get AmbiguousPrefix error:
	let result = envflag::key("PORT").default(0u16).get();
	println!("Ambiguous lookup result: {result:?}");
}
