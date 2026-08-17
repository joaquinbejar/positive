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

# Run tests in the default configuration.
.PHONY: test
test:
	LOGLEVEL=WARN cargo test

# Run the full feature matrix required by rules/global_rules.md.
#
# The three configurations are separate runs on purpose: the positivity
# invariant differs between them, and cfg-gated items only exist in their own
# configuration. An --all-features run cannot stand in for the default one,
# which is why the coverage job is not a substitute for this target.
.PHONY: test-matrix
test-matrix:
	LOGLEVEL=WARN cargo test --all-features
	LOGLEVEL=WARN cargo test --no-default-features
	LOGLEVEL=WARN cargo test --features non-zero

# Reject the patterns rules/global_rules.md bans from production code:
# .unwrap() and .expect() outside #[cfg(test)].
#
# Three exclusions, each narrow and visible:
#   - comment lines, because an example inside /// is documentation;
#   - everything from the first #[cfg(test)] in a file onwards, which is how
#     this crate places its test modules;
#   - lines carrying an explicit `// scan-banned: allow -- <reason>` marker, so
#     an exemption is recorded at the site rather than hidden in this file.
#
# `unsafe` is enforced separately and more strongly by #![forbid(unsafe_code)]
# in src/lib.rs, which is a compile error rather than a grep.
.PHONY: scan-banned
scan-banned:
	@found=$$(for f in $$(find src -name '*.rs'); do \
		awk -v file="$$f" '/#\[cfg\(test\)\]/ { exit } { print file ":" NR ":" $$0 }' "$$f"; \
	done \
		| grep -E '\.unwrap\(\)|\.expect\(' \
		| grep -v -E ':[0-9]+:[[:space:]]*(///|//!|//|\*)' \
		| grep -v 'scan-banned: allow' || true); \
	if [ -n "$$found" ]; then \
		echo "Banned patterns found in production code:"; \
		echo "$$found"; \
		exit 1; \
	fi; \
	echo "OK: no .unwrap()/.expect() in production code"
	@$(MAKE) --no-print-directory scan-indexing

# Unchecked `[]` indexing is the third banned panic source, and a grep cannot
# tell `arr[i]` from a macro or a type parameter. Clippy can, so the check runs
# as a lint rather than a scan. It is restricted to `--lib`, because the ban
# covers `src/` only: tests and benches index freely by design. Each feature
# configuration is linted separately, since cfg-gated code is only visible in
# its own.
.PHONY: scan-indexing
scan-indexing:
	cargo clippy --lib --all-features -- -D clippy::indexing_slicing
	cargo clippy --lib --no-default-features -- -D clippy::indexing_slicing
	cargo clippy --lib --features non-zero -- -D clippy::indexing_slicing
	@echo "OK: no unchecked indexing in production code"

# Documentation must build with zero warnings.
.PHONY: doc-check
doc-check:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# The README is generated from the crate docs. Verify rather than rewrite, so
# the check cannot pass by silently changing a tracked file.
.PHONY: readme-check
readme-check: check-cargo-readme
	@cargo readme > /tmp/positive-readme-check.md
	@diff -u README.md /tmp/positive-readme-check.md \
		|| (echo "README.md is out of date; run 'make readme'" && exit 1)
	@echo "OK: README.md matches the crate docs"

# Release build must be warning-free.
.PHONY: release-check
release-check:
	RUSTFLAGS="-D warnings" cargo build --release

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

# The quality gate. Mirrors rules/global_rules.md exactly, and is what CI runs.
#
# Every target below is read-only: nothing here formats, fixes or regenerates a
# tracked file. Mutating helpers live under explicitly named targets — fmt,
# fix, lint-fix, readme — and are never invoked by a gate.
.PHONY: check
check: fmt-check lint lint-strict test-matrix release-check doc-check readme-check scan-banned audit
	@echo ""
	@echo "All quality gates passed."

# Run the project
.PHONY: run
run:
	cargo run

.PHONY: fix
fix:
	cargo fix --allow-staged --allow-dirty

# Pre-push is the gate, not a fixer. It used to run `cargo fix`, `clippy --fix`
# and regenerate the README, so a command named as a check mutated the worktree
# and could turn a failure into a silent edit. Use `make fixup` for that.
.PHONY: pre-push
pre-push: check

# Explicitly mutating convenience target. Never invoked by a gate.
.PHONY: fixup
fixup: fix fmt lint-fix readme

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
