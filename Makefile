.PHONY: test bench fmt release

SRCS = $(shell find src -name '*.rs')

test: cargo-test integration-ruby integration-rust

cargo-test:
	cargo test

integration-ruby:
	time bash test.sh

integration-rust: t
	time env COMMAND=target/debug/t bash test.sh

t: target/debug/t

target/debug/t: $(SRCS) Cargo.toml Cargo.lock
	cargo build

bench: target/debug/bench-parse target/debug/bench-sum
	bash bench.sh

fmt:
	rustfmt -l $(SRCS)

target/debug/%: $(SRCS)
	cargo build --bin $(notdir $@)
