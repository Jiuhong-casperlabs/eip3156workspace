#![no_main]
#![no_std]

use casper_contract::{
    self,
    contract_api::{runtime, storage},
};
use casper_types::{runtime_args, ContractPackageHash, Key, RuntimeArgs, U256};

#[no_mangle]
fn call() {
    let lend_package_str =
        "contract-package-wasm841a2dbe42b87370842bf89999210630664a729b6ae85eceb3cdd57a1711777a";

    let lend_package = ContractPackageHash::from_formatted_str(lend_package_str).unwrap();

    let token_package_str =
        "contract-package-wasm841a2dbe42b87370842bf89999210630664a729b6ae85eceb3cdd57a1711777a";
    // "contract-package-wasm3ef68bed379e1b1f23c9764cee06eb9317ffbcd7ceeb0f83c25097b69bdb0c61";

    let token_package = ContractPackageHash::from_formatted_str(token_package_str).unwrap();

    let token = Key::from(token_package);
    let flash_fee: U256 = runtime::call_versioned_contract(
        lend_package,
        None,
        "flash_fee",
        runtime_args! {
            "token" => token,
            "amount" => U256::from(999999999)
        },
    );

    runtime::put_key("flashfee", storage::new_uref(flash_fee).into());
}
