SRCS = $(shell find src -name '*.rs')

.PHONY: default
default: test release

.PHONY: release
release: target/release/t

.PHONY: install
install: release
	cp target/release/t ~/.bin/t

.PHONY: test
test: cargo-test integration-rust

.PHONY: bench
bench: target/debug/bench-parse target/debug/bench-sum
	bash bench.sh

.PHONY: cargo-test
cargo-test:
	cargo test

.PHONY: integration-rust
integration-rust: target/debug/t
	time env COMMAND=target/debug/t bash test.sh

target/debug/t: $(SRCS) Cargo.toml Cargo.lock
	cargo build

target/release/t: $(SRCS) Cargo.toml Cargo.lock
	cargo build --release

.PHONY: clippy
clippy:
	cargo clippy

target/debug/%: $(SRCS) Cargo.toml Cargo.lock
	cargo build --bin $(notdir $@)
