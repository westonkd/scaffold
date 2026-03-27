.PHONY: help build dev watch test fmt clippy release clean clean-volumes build-authorizer

help: ## List available targets
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*##"}; {printf "  %-16s %s\n", $$1, $$2}'

build: ## Build the dev Docker image
	docker compose build dev

dev: ## Open interactive shell in dev container
	docker compose run --rm dev

watch: ## Start cargo-watch daemon
	docker compose run --rm watch

test: ## Run cargo test in container
	docker compose run --rm dev cargo test

fmt: ## Run cargo fmt in container
	docker compose run --rm dev cargo fmt

clippy: ## Run cargo clippy in container
	docker compose run --rm dev cargo clippy --all-targets -- -D warnings

release: ## Build release image and copy static binary to dist/
	docker compose build release
	docker compose run --rm release

clean: ## Run cargo clean and remove dist/ binary
	docker compose run --rm dev cargo clean
	rm -f dist/hoist

clean-volumes: ## Remove Docker volumes (nukes cargo caches)
	docker compose down -v

build-authorizer: ## Build the Lambda authorizer zip (required before terraform plan with enable_apigw=true)
	docker build \
		--target export \
		--output type=local,dest=lambda/authorizer/dist \
		lambda/authorizer
