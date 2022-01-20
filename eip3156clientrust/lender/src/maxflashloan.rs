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
        "contract-package-wasm96053169b397360449b4de964200be449594ca93f252153f0a679b804e214a54";

    let lend_package = ContractPackageHash::from_formatted_str(lend_package_str).unwrap();

    let token_package_str =
        "contract-package-wasm17202d448a32af52252a21c8296c9562c10a1f3da69efc5a5d01678aac753b7e";
    // "contract-package-wasm3ef68bed379e1b1f23c9764cee06eb9317ffbcd7ceeb0f83c25097b69bdb0c61";

    let token_package = ContractPackageHash::from_formatted_str(token_package_str).unwrap();

    let token = Key::from(token_package);
    let flash_fee: U256 = runtime::call_versioned_contract(
        lend_package,
        None,
        "max_flash_loan",
        runtime_args! {
            "token" => token,
            "amount" => U256::from(999999999)
        },
    );

    runtime::put_key("maxflashloan", storage::new_uref(flash_fee).into());
}
