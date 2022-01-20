#![no_std]
#![no_main]

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{string::String, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::{
    bytesrepr::ToBytes, runtime_args, ContractPackageHash, Key, RuntimeArgs, URef, U256,
};

fn make_dictionary_item_key(owner: Key, spender: Key) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut owner.to_bytes().unwrap_or_revert());
    preimage.append(&mut spender.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

#[no_mangle]
pub extern "C" fn call() {
    let owner_package_hash = ContractPackageHash::from_formatted_str(
        "contract-package-wasm43a14f839e948ac99ef19ff0a20cc9c96d96a5ef56a9df79374e6d6dad4813ce", //borrwer
    )
    .unwrap();
    let owner = Key::from(owner_package_hash);

    let spender_package_hash = ContractPackageHash::from_formatted_str(
        "contract-package-wasmd2f51fc9ab2f63559842c072ee1e100149d6538774542d91484ad3005b782300", //lender
    )
    .unwrap();
    let spender = Key::from(spender_package_hash);

    let dictionary_item_key = make_dictionary_item_key(owner, spender);

    runtime::put_key(
        "allowancekey",
        storage::new_uref(dictionary_item_key).into(),
    );
}
