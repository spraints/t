.PHONY: test bench fmt

SRCS = $(shell find src -name '*.rs')

test:
	cargo test
	cargo build
	bash test.sh
	env COMMAND=target/debug/t bash test.sh

bench: target/debug/bench-parse target/debug/bench-sum
	bash bench.sh

fmt:
	rustfmt -l $(SRCS)

target/debug/%: $(SRCS)
	cargo build --bin $(notdir $@)
