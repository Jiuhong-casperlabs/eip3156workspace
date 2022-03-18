prepare:
	rustup target add wasm32-unknown-unknown

.PHONY:	build-contracts
build-contracts:
	cargo build --release -p borrower --target wasm32-unknown-unknown
	cargo build --release -p lender --target wasm32-unknown-unknown
	cargo build --release -p minter --target wasm32-unknown-unknown
	cargo build --release -p eip3156-test-call --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/borrower.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/lender.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/minter.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/eip3156_test_call.wasm 2>/dev/null | true

test-only:
	cd tests/testing && cargo test

test: build-contracts
	mkdir -p tests/testing/wasm
	cp target/wasm32-unknown-unknown/release/borrower.wasm tests/testing/wasm
	cp target/wasm32-unknown-unknown/release/lender.wasm tests/testing/wasm
	cp target/wasm32-unknown-unknown/release/eip3156_test_call.wasm tests/testing/wasm
	cd tests/testing && cargo test -- --nocapture

clippy:
	cargo clippy --all-targets -- -D warnings


check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all

clean:
	rm -rf tests/testing/wasm
	cargo clean