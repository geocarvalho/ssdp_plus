# SSDP+ Rust crate — convenience targets (run from this directory).

.PHONY: default fmt clippy test bench build-release check run-matrix run-alon

CRATE_DIR := .

default: test

fmt:
	cd $(CRATE_DIR) && cargo fmt --all

clippy:
	cd $(CRATE_DIR) && cargo clippy --all-targets -- -D warnings

test:
	cd $(CRATE_DIR) && cargo test

bench:
	cd $(CRATE_DIR) && cargo bench --bench ssdp_search

build-release:
	cd $(CRATE_DIR) && cargo build --release

# Quality gate: format, lint, test
check: fmt clippy test

MATRIX := $(CRATE_DIR)/data/matrixBinaria-Global-100-p.csv
ALON := $(CRATE_DIR)/data/alon-clean50-pn-width-2.CSV

run-matrix:
	cd $(CRATE_DIR) && cargo run --release -- \
		--dataset $(MATRIX) \
		--target-attr class \
		--target-value p \
		-k 5 --seed 0 --cache-size 2 --min-similarity 0.10

run-alon:
	cd $(CRATE_DIR) && cargo run --release -- \
		--dataset $(ALON) \
		--target-attr y \
		--target-value p \
		-k 5 --seed 0 --cache-size 2 --min-similarity 0.10
