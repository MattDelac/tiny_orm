DESCRIBE           := $(shell git describe --match "v*" --always --tags)
DESCRIBE_PARTS     := $(subst -, ,$(DESCRIBE))
VERSION_TAG        := $(word 1,$(DESCRIBE_PARTS))
VERSION            := $(subst v,,$(VERSION_TAG))
VERSION_PARTS      := $(subst ., ,$(VERSION))
MAJOR              := $(word 1,$(VERSION_PARTS))
MINOR              := $(word 2,$(VERSION_PARTS))
MICRO              := $(word 3,$(VERSION_PARTS))

test:
	cargo test --lib --tests --features sqlite
	cargo run --example sqlite --features sqlite
	cargo run --example postgres --features postgres
	cargo run --example mysql --features mysql

define validate_version
	@if echo "$(1)" | grep -qE "^[0-9]\.[0-9]{1,2}\.[0-9]{1,2}$$"; then \
		echo "Version '$(1)' is valid"; \
	else \
		echo "Version must be in format v0.1.2 but is instead '$(1)'"; \
		exit 1; \
	fi
endef

define update_version
	@NEXT_VERSION="$(1)" && \
	CURRENT_VERSION="$(VERSION)" && \
	perl -pi -e 's/version = "'"$$CURRENT_VERSION"'"/version = "'"$$NEXT_VERSION"'"/' Cargo.toml && \
	perl -pi -e 's/tiny-orm-macros = \{ version = "'"$$CURRENT_VERSION"'"/tiny-orm-model = { version = "'"$$NEXT_VERSION"'"/' Cargo.toml && \
	perl -pi -e 's/tiny-orm-macros = \{ version = "'"$$CURRENT_VERSION"'"/tiny-orm-macros = { version = "'"$$NEXT_VERSION"'"/' Cargo.toml && \
	perl -pi -e 's/version = "'"$$CURRENT_VERSION"'"/version = "'"$$NEXT_VERSION"'"/' tiny-orm-model/Cargo.toml && \
	perl -pi -e 's/version = "'"$$CURRENT_VERSION"'"/version = "'"$$NEXT_VERSION"'"/' tiny-orm-macros/Cargo.toml && \
	perl -pi -e 's/tiny-orm-macros = \{ version = "'"$$CURRENT_VERSION"'"/tiny-orm-model = { version = "'"$$NEXT_VERSION"'"/' tiny-orm-macros/Cargo.toml && \
	perl -pi -e 's/tiny-orm = \{version = "'"$$CURRENT_VERSION"'"/tiny-orm = {version = "'"$$NEXT_VERSION"'"/' README.md && \
	perl -pi -e 's/tiny-orm\/'"$$CURRENT_VERSION"'\/tiny_orm/tiny-orm\/'"$$NEXT_VERSION"'\/tiny_orm/' README.md

	@echo "Version updated to $(1) from $(VERSION) in all files"
endef

release:
ifndef INCREMENT
	@echo "Please provide:"
	@echo "  - An increment type: make release INCREMENT=patch|minor|major"
	exit 1
endif
ifeq ($(INCREMENT),patch)
	@$(eval NEXT_VERSION := $(MAJOR).$(MINOR).$(shell expr $(MICRO) + 1))
	@echo "Next version will be $(NEXT_VERSION) from $(VERSION)"
	$(call validate_version,$(NEXT_VERSION))
	$(call update_version,$(NEXT_VERSION))
else ifeq ($(INCREMENT),minor)
	@$(eval NEXT_VERSION := $(MAJOR).$(shell expr $(MINOR) + 1).0)
	@echo "Next version will be $(NEXT_VERSION) from $(VERSION)"
	$(call validate_version,$(NEXT_VERSION))
	$(call update_version,$(NEXT_VERSION))
else ifeq ($(INCREMENT),major)
	@$(eval NEXT_VERSION := $(shell expr $(MAJOR) + 1).0.0)
	@echo "Next version will be $(NEXT_VERSION) from $(VERSION)"
	$(call validate_version,$(NEXT_VERSION))
	$(call update_version,$(NEXT_VERSION))
else
	@echo "Invalid increment type $(INCREMENT)"; \
	exit 1;
endif

.PHONY: test release
