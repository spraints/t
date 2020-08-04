.PHONY: test fmt

SRCS = $(shell find src -name '*.rs')

test:
	cargo test
	cargo build
	bash test.sh
	env COMMAND=target/debug/t bash test.sh

fmt:
	rustfmt -l $(SRCS)
