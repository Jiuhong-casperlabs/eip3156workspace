prepare:
	rustup target add wasm32-unknown-unknown

.PHONY:	build-contracts
build-contracts:
	cargo build --release -p borrower --target wasm32-unknown-unknown
	cargo build --release -p lender --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/borrower.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/lender.wasm 2>/dev/null | true

test-only:
	cd tests/testborrower && cargo test

test: build-contracts
	mkdir -p tests/testborrower/wasm
	cp target/wasm32-unknown-unknown/release/borrower.wasm tests/testborrower/wasm
	cp target/wasm32-unknown-unknown/release/lender.wasm tests/testborrower/wasm
	cd tests/testborrower && cargo test -- --nocapture

clippy:
	cargo clippy --all-targets -- -D warnings


check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all

clean:
	cargo clean