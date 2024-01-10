.PHONY: build lint test clean help

CRATES := rsa_contract rsa_circuit client

build: ## Build all the project components
	@for crate in $(CRATES); do \
		cargo build --release --manifest-path ./$$crate/Cargo.toml || exit 1; \
	done
	@cargo contract build --release --manifest-path ./rsa_contract/Cargo.toml || exit 1;

lint: ## Run the linter
	@for crate in $(CRATES); do \
		cargo +nightly fmt --manifest-path ./$$crate/Cargo.toml || exit 1; \
		cargo clippy --release --manifest-path ./$$crate/Cargo.toml -- -D warnings || exit 1; \
	done

test: ## Run tests
	@for crate in $(CRATES); do \
		cargo test --release --manifest-path ./$$crate/Cargo.toml -- --show-output || exit 1; \
	done

clean: ## Clean all the build files
	@for crate in $(CRATES); do \
		cargo clean --manifest-path ./$$crate/Cargo.toml || exit 1; \
	done

help: ## Displays this help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[1;36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[1;36m%-25s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
