.PHONY: build lint test clean help

build: ## Build the project
	cargo contract build --release --manifest-path ./rsa_contract/Cargo.toml

lint: ## Run the linter
	cargo +nightly fmt --manifest-path ./rsa_contract/Cargo.toml
	cargo clippy --release --manifest-path ./rsa_contract/Cargo.toml -- -D warnings

test: ## Run tests
	cargo test --release --manifest-path ./rsa_contract/Cargo.toml -- --show-output

clean: ## Clean all the build files
	cargo clean --manifest-path ./rsa_contract/Cargo.toml

help: ## Displays this help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[1;36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z0-9_-]+:.*?##/ { printf "  \033[1;36m%-25s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
