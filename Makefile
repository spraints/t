.PHONY: test bench release clippy

SRCS = $(shell find src -name '*.rs')

test: cargo-test rspec integration-ruby integration-rust

cargo-test:
	cargo test

rspec:
	bin/rspec

integration-ruby:
	time env COMMAND=bin/t bash test.sh

integration-rust: target/debug/t
	time env COMMAND=target/debug/t bash test.sh

t: target/debug/t

release: target/release/t

target/debug/t: $(SRCS) Cargo.toml Cargo.lock
	cargo build

target/release/t: $(SRCS) Cargo.toml Cargo.lock
	cargo build --release

bench: target/debug/bench-parse target/debug/bench-sum
	bash bench.sh

clippy:
	cargo clippy

target/debug/%: $(SRCS)
	cargo build --bin $(notdir $@)
