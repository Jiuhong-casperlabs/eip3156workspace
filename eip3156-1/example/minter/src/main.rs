#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, DECIMALS_RUNTIME_ARG_NAME,
        NAME_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME, SYMBOL_RUNTIME_ARG_NAME, TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    },
    Address, ERC20,
};
use casper_types::{CLValue, U256};

#[no_mangle]
pub extern "C" fn name() {
    let name = ERC20::default().name();
    runtime::ret(CLValue::from_t(name).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn symbol() {
    let symbol = ERC20::default().symbol();
    runtime::ret(CLValue::from_t(symbol).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn decimals() {
    let decimals = ERC20::default().decimals();
    runtime::ret(CLValue::from_t(decimals).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn total_supply() {
    let total_supply = ERC20::default().total_supply();
    runtime::ret(CLValue::from_t(total_supply).unwrap_or_revert());
}
// EIP3156 start
#[no_mangle]
pub extern "C" fn loan_fee() {
    let loan_fee = ERC20::default().loan_fee();
    runtime::ret(CLValue::from_t(loan_fee).unwrap_or_revert());
}
// EIP3156 end

#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Address = runtime::get_named_arg(ADDRESS_RUNTIME_ARG_NAME);
    let balance = ERC20::default().balance_of(address);
    runtime::ret(CLValue::from_t(balance).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    ERC20::default()
        .transfer(recipient, amount)
        .unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn approve() {
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);

    ERC20::default().approve(spender, amount).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn allowance() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let spender: Address = runtime::get_named_arg(SPENDER_RUNTIME_ARG_NAME);
    let val = ERC20::default().allowance(owner, spender);
    runtime::ret(CLValue::from_t(val).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let recipient: Address = runtime::get_named_arg(RECIPIENT_RUNTIME_ARG_NAME);
    let amount: U256 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    ERC20::default()
        .transfer_from(owner, recipient, amount)
        .unwrap_or_revert();
}
// EIP3156 start
#[no_mangle]
fn flash_fee() {
    let token: Address = runtime::get_named_arg("token");
    let amount: U256 = runtime::get_named_arg("amount");
    let flash_fee = ERC20::default().flash_fee(token, amount).unwrap_or_revert();
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
    let success = ERC20::default()
        .flash_loan(receiver, token, amount, data)
        .unwrap_or_revert();
    runtime::ret(CLValue::from_t(success).unwrap_or_revert());
}

#[no_mangle]
fn max_flash_loan() {
    let max_loan = ERC20::default().max_flash_loan();
    runtime::ret(CLValue::from_t(max_loan).unwrap_or_revert());
}
// EIP3156 end
#[no_mangle]
fn call() {
    let name: String = runtime::get_named_arg(NAME_RUNTIME_ARG_NAME);
    let symbol: String = runtime::get_named_arg(SYMBOL_RUNTIME_ARG_NAME);
    // EIP3156 start
    let fee: U256 = runtime::get_named_arg("fee");
    // EIP3156 end
    let decimals = runtime::get_named_arg(DECIMALS_RUNTIME_ARG_NAME);
    let total_supply = runtime::get_named_arg(TOTAL_SUPPLY_RUNTIME_ARG_NAME);

    let _token = ERC20::install(
        name,
        symbol,
        // EIP3156 start
        fee,
        // EIP3156 end
        decimals,
        total_supply,
    )
    .unwrap_or_revert();
}
