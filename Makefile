test:
	cargo test --lib --tests --features sqlite
	cargo run --example sqlite --features sqlite
	cargo run --example postgres --features postgres
	cargo run --example mysql --features mysql

release:
	@if [ -n "$(VERSION)" ]; then \
		$(call validate_version,$(VERSION)) \
		$(call update_version,$(VERSION)); \
	elif [ -n "$(INCREMENT)" ]; then \
		CURRENT_VERSION="v$(call get_current_version)"; \
		NEW_VERSION="$(call increment_version,$$CURRENT_VERSION,$(INCREMENT))"; \
		$(call validate_version,$$NEW_VERSION) \
		$(call update_version,$$NEW_VERSION); \
	else \
		echo "Please provide either:"; \
		echo "  - A specific version: make release VERSION=v0.1.2"; \
		echo "  - An increment type: make release INCREMENT=patch|minor|major"; \
		exit 1; \
	fi

define validate_version
	@if ! echo "$(1)" | grep -qE "^v[0-9]\.[0-9]{1,2}\.[0-9]{1,2}$$"; then \
		echo "Version must be in format v0.1.2"; \
		exit 1; \
	fi
endef

define update_version
	@VERSION_NUM=$$(echo "$(1)" | sed 's/^v//') && \
	sed -i.bak "s/^version = \".*\"/version = \"$$VERSION_NUM\"/" Cargo.toml && \
	sed -i.bak "s/^version = \".*\"/version = \"$$VERSION_NUM\"/" tiny-orm-macros/Cargo.toml && \
	sed -i.bak "s/tiny-orm-macros = {version = \"[^\"]*\"/tiny-orm-macros = {version = \"$$VERSION_NUM\"/" Cargo.toml && \
	sed -i.bak "s/tiny-orm = {version = \"[^\"]*\"/tiny-orm = {version = \"$$VERSION_NUM\"/" README.md && \
	rm Cargo.toml.bak README.md.bak && \
	echo "Version updated to $(1) in Cargo.toml and README.md"
endef

define get_current_version
	$(shell grep '^version = ' Cargo.toml | head -1 | cut -d '"' -f2)
endef

define increment_version
	$(shell echo "$(1)" | awk -F. -v type=$(2) '{ \
		if (type == "patch") $$NF++; \
		else if (type == "minor") { $$2++; $$3=0; } \
		else if (type == "major") { $$1++; $$2=0; $$3=0; } \
		print "v"$$1"."$$2"."$$3 \
	}')
endef
