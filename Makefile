test:
	cargo test --lib --features sqlite

	cargo run --example postgres --features postgres
	cd examples/sqlite && cargo run --example sqlite --features sqlite
