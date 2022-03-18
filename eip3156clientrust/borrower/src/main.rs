#![no_main]

use casper_contract::contract_api::{runtime, storage};

#[no_mangle]
fn call() {
    runtime::put_key("hello", storage::new_uref("world!").into());
}
