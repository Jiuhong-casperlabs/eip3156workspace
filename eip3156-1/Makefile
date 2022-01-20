prepare:
	rustup target add wasm32-unknown-unknown

.PHONY:	build-contracts
build-contracts:
	cargo build --release -p br-contract --target wasm32-unknown-unknown
	cargo build --release -p ld-contract --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/br-contract.wasm 2>/dev/null | true
	wasm-strip target/wasm32-unknown-unknown/release/ld-contract.wasm 2>/dev/null | true


clippy:
	cargo clippy --all-targets -- -D warnings
	# cargo clippy --all-targets -p br-contract --target wasm32-unknown-unknown -- -D warnings


check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all

clean:
	cargo clean