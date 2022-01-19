#![no_std]
#![no_main]

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use casper_contract::contract_api::{runtime, storage};

use casper_types::{runtime_args, ContractHash, Key, RuntimeArgs, U256};

#[no_mangle]
pub extern "C" fn call() {
    let erc20_contract_hash =
        "contract-fef68ea3f80526a1989f7a431986a599d20a2e8e5585e1f6adcfb2eba5c3d4be";
    let contract_hash = ContractHash::from_formatted_str(erc20_contract_hash).unwrap();

    let raw_address =
    "account-hash-a66aa5ab61b26cd4178cba3fa5657652013f57689eb25e111f3aa974443591b1"
    // "account-hash-2293223427D59eBB331aC2221c3fcd1b3656a5Cb72BE924A6CdC9d52CdB6dB0F" jdk2
    ;
    let address = Key::from_formatted_str(
        //
        raw_address,
    )
    .unwrap();

    let args = runtime_args! {
        "address" => address,

    };

    let balances: U256 = runtime::call_contract(contract_hash, "balance_of", args);
    runtime::put_key("mybalance", storage::new_uref(balances).into());
}
