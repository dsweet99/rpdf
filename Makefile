.PHONY: all build install test test-fast test-slow

all: build

build:
	cargo build --release

install:
	cargo install --path . --force

test:
	cargo test
	RPDF_RUN_SLOW=1 cargo test --test slow -- --ignored

test-fast:
	cargo test

test-slow:
	RPDF_RUN_SLOW=1 cargo test --test slow -- --ignored
