#![no_std]
#![no_main]

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use casper_contract::contract_api::{runtime, storage};

use casper_types::{runtime_args, ContractPackageHash, Key, RuntimeArgs, U256};

#[no_mangle]
pub extern "C" fn call() {
    let contractpackagehash = ContractPackageHash::from_formatted_str(
        "contract-package-wasm2c4ea63b2ece508ba7a3f68ab79e6bfc76a3195012d72c3cb15df72f5f438b1d", //erc20
                                                                                                 // "contract-package-wasmf27e4d26f43d64a9e0688f0d90f4c129e50f930b0d46416af1f1c9a18f957dbb", //erc20_1
    )
    .unwrap();
    // let raw_address =
    // "hash-d881151cf7fd63a668889b9dfc3975339e72e01c947a9b9ff93add30afa2a6d4" //LENDER
    // ;
    let raw_address = "hash-d881151cf7fd63a668889b9dfc3975339e72e01c947a9b9ff93add30afa2a6d4";
    let address = Key::from_formatted_str(raw_address).unwrap();

    // let balances: U256 = runtime::call_contract(contractpackagehash, "balance_of", args);
    let balance: U256 = runtime::call_versioned_contract(
        contractpackagehash,
        None,
        "balance_of",
        runtime_args! {
            "address" => address,

        },
    );

    runtime::put_key("mybalance", storage::new_uref(balance).into());
}
