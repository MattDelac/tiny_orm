test:
	cargo test --lib --tests --features sqlite
	cargo run --example postgres --features postgres
	cargo run --example sqlite --features sqlite

release:
	@if [ -z "$(VERSION)" ]; then \
		echo "Please provide a version number: make release VERSION=v0.1.2"; \
		exit 1; \
	fi
	@if ! echo "$(VERSION)" | grep -qE "^v[0-9]\.[0-9]{1,2}\.[0-9]{1,2}$$"; then \
		echo "Version must be in format v0.1.2"; \
		exit 1; \
	fi
	@VERSION_NUM=$$(echo "$(VERSION)" | sed 's/^v//') && \
	sed -i.bak "s/^version = \".*\"/version = \"$$VERSION_NUM\"/" Cargo.toml && \
	sed -i.bak "s/tiny-orm = {version = \"[^\"]*\"/tiny-orm = {version = \"$$VERSION_NUM\"/" README.md && \
	rm Cargo.toml.bak README.md.bak && \
	echo "Version updated to $(VERSION) in Cargo.toml and README.md"
