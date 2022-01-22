#![no_main]

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_erc20::Address;
use casper_types::{CLValue, U256};
use flashborrower::EIP3156BORROWER;

#[no_mangle]
fn on_flash_loan() {
    let initiator: Address = runtime::get_named_arg("initiator");
    let token: Address = runtime::get_named_arg("token");
    let amount: U256 = runtime::get_named_arg("amount");
    let fee: U256 = runtime::get_named_arg("fee");
    let data: String = runtime::get_named_arg("data");
    let call_result = EIP3156BORROWER::default()
        .on_flash_loan(initiator, token, amount, fee, data)
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(call_result).unwrap_or_revert());
}

#[no_mangle]
fn flash_borrow() {
    let token: Address = runtime::get_named_arg("token");
    let amount: U256 = runtime::get_named_arg("amount");
    EIP3156BORROWER::default()
        .flash_borrow(token, amount)
        .unwrap_or_revert();
}

#[no_mangle]
fn call() {
    let lender_address: Address = runtime::get_named_arg("lender_address");
    let _borrower = EIP3156BORROWER::install(lender_address).unwrap_or_revert();
}
