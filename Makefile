# Makefile for common tasks in a Rust project
# Detect current branch
CURRENT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD)


# Default target
.PHONY: all
all: test fmt lint build

# Build the project
.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

# Run tests
.PHONY: test
test:
	LOGLEVEL=WARN cargo test

# Format the code
.PHONY: fmt
fmt:
	cargo +stable fmt --all

# Check formatting
.PHONY: fmt-check
fmt-check:
	cargo +stable fmt --check

# Run Clippy for linting
.PHONY: lint
lint:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

.PHONY: lint-fix
lint-fix: 
	cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged --workspace -- -D warnings

# Strict public-API lints from rules/global_rules.md, beyond the default
# clippy set: every public item documented, every Result API with an Errors
# section, every panicking API with a Panics section, and #[must_use] where the
# rules require it. Run under each feature configuration, since cfg-gated items
# are only visible in their own.
.PHONY: lint-strict
lint-strict:
	cargo clippy --all-targets --all-features -- \
		-D warnings \
		-D clippy::must_use_candidate \
		-D clippy::missing_errors_doc \
		-D clippy::missing_panics_doc \
		-D missing_docs
	cargo clippy --all-targets --no-default-features -- \
		-D warnings \
		-D clippy::must_use_candidate \
		-D clippy::missing_errors_doc \
		-D clippy::missing_panics_doc \
		-D missing_docs
	cargo clippy --all-targets --features non-zero -- \
		-D warnings \
		-D clippy::must_use_candidate \
		-D clippy::missing_errors_doc \
		-D clippy::missing_panics_doc \
		-D missing_docs

# Clean the project
.PHONY: clean
clean:
	cargo clean

# Security audit — mirrors .github/workflows/audit.yml exactly.
# Ignored advisories, their reachability rationale, owner and review date live
# in .cargo/audit.toml, which is the single source of truth for both local and
# CI runs.
.PHONY: audit
audit: check-cargo-audit
	cargo audit --deny warnings

.PHONY: check-cargo-audit
check-cargo-audit:
	@command -v cargo-audit > /dev/null || (echo "Installing cargo-audit..."; cargo install cargo-audit --no-default-features)

# Pre-push checks
.PHONY: check
check: test fmt-check lint

# Run the project
.PHONY: run
run:
	cargo run

.PHONY: fix
fix:
	cargo fix --allow-staged --allow-dirty

.PHONY: pre-push
pre-push: fix fmt lint-fix test readme doc

.PHONY: doc
doc:
	cargo clippy -- -W missing-docs

.PHONY: doc-open
doc-open:
	cargo doc --open

.PHONY: publish
publish: readme
	cargo login ${CARGO_REGISTRY_TOKEN}
	cargo package
	cargo publish

# Note: cargo-tarpaulin reports line coverage only — its engines do not
# instrument branches or conditions, which is why the report shows 0 branches
# rather than a low number. Boundary behaviour is covered explicitly by
# tests/boundary_matrix.rs instead of inferred from the line percentage.
.PHONY: coverage
coverage:
	export LOGLEVEL=WARN
	cargo install cargo-tarpaulin
	mkdir -p coverage
	cargo tarpaulin --verbose --all-features --workspace --timeout 0 --out Xml --output-dir coverage

.PHONY: coverage-html
coverage-html:
	export LOGLEVEL=WARN
	cargo install cargo-tarpaulin
	mkdir -p coverage
	cargo tarpaulin --color Always --engine llvm --tests --all-targets --all-features --workspace --timeout 0 --out Html --output-dir coverage

.PHONY: open-coverage
open-coverage:
	open tarpaulin-report.html

# Rule to show git log
git-log:
	@if [ "$(CURRENT_BRANCH)" = "HEAD" ]; then \
		echo "You are in a detached HEAD state. Please check out a branch."; \
		exit 1; \
	fi; \
	echo "Showing git log for branch $(CURRENT_BRANCH) against main:"; \
	git log main..$(CURRENT_BRANCH) --pretty=full

.PHONY: create-doc
create-doc:
	cargo doc --no-deps --document-private-items

.PHONY: readme
readme: check-cargo-readme create-doc
	cargo readme > README.md

.PHONY: check-cargo-readme
check-cargo-readme:
	@command -v cargo-readme > /dev/null || (echo "Installing cargo-readme..."; cargo install cargo-readme)

.PHONY: check-spanish
check-spanish:
	@rg -n --pcre2 -e '^\s*(//|///|//!|#|/\*|\*).*?[áéíóúÁÉÍÓÚñÑ¿¡]' \
    	    --glob '!target/*' \
    	    --glob '!**/*.png' \
    	    . || (echo "❌  Spanish comments found"; exit 1)

.PHONY: check-cargo-criterion
check-cargo-criterion:
	@command -v cargo-criterion > /dev/null || (echo "Installing cargo-criterion..."; cargo install cargo-criterion)

.PHONY: bench
bench: check-cargo-criterion
	cargo criterion --output-format=quiet

.PHONY: bench-show
bench-show:
	open target/criterion/report/index.html

.PHONY: bench-save
bench-save: check-cargo-criterion
	cargo criterion --output-format quiet --history-id v0.4.0 --history-description "Version 0.4.0 baseline"

.PHONY: bench-compare
bench-compare: check-cargo-criterion
	cargo criterion --output-format verbose

.PHONY: bench-json
bench-json: check-cargo-criterion
	cargo criterion --message-format json

.PHONY: bench-clean
bench-clean:
	rm -rf target/criterion


.PHONY: workflow-coverage
workflow-coverage:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job code_coverage_report \
       -P ubuntu-latest=catthehacker/ubuntu:latest \
       --privileged

.PHONY: workflow-build
workflow-build:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job build \
       -P ubuntu-latest=catthehacker/ubuntu:latest

.PHONY: workflow-lint
workflow-lint:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job lint

.PHONY: workflow-test
workflow-test:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job run_tests

.PHONY: workflow
workflow: workflow-build workflow-lint workflow-test workflow-coverage
