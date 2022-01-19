prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p lender --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/flash_fee.wasm 2>/dev/null | true
	cargo build --release -p erc20 --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/getbalance.wasm 2>/dev/null | true
	

clippy:
	cargo clippy --all-targets -- -D warnings
	# cargo clippy --all-targets -p br-contract --target wasm32-unknown-unknown -- -D warnings


check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all

clean:
	cargo clean