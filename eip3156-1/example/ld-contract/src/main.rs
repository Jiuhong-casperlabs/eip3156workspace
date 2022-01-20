#![no_main]

use flashlender::EIP3156LENDER;

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_erc20::Address;
use casper_types::{CLValue, U256};

#[no_mangle]
fn flash_fee() {
    let token: Address = runtime::get_named_arg("token");
    let amount: U256 = runtime::get_named_arg("amount");
    let flash_fee = EIP3156LENDER::default()
        .flash_fee(token, amount)
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(flash_fee).unwrap_or_revert());
}

#[no_mangle]
fn flash_loan() {
    // receiver: Address,
    // token: Address,
    // amount: U256,
    // data: Bytes,
    let receiver: Address = runtime::get_named_arg("receiver");
    let token: Address = runtime::get_named_arg("token");
    let amount: U256 = runtime::get_named_arg("amount");
    let data: String = runtime::get_named_arg("data");
    let success = EIP3156LENDER::default()
        .flash_loan(receiver, token, amount, data)
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(success).unwrap_or_revert());
}

#[no_mangle]
fn max_flash_loan() {
    let token: Address = runtime::get_named_arg("token");
    let max_loan = EIP3156LENDER::default().max_flash_loan(token);
    runtime::ret(CLValue::from_t(max_loan).unwrap_or_revert());
}

#[no_mangle]
fn call() {
    let initial_supportted_tokens: Vec<(Address, U256)> =
        runtime::get_named_arg("initial_supportted_tokens");
    let _lender = EIP3156LENDER::install(initial_supportted_tokens).unwrap_or_revert();
}
