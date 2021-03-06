#![no_std]
#![no_main]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec,
};

use casper_contract::{
    self,
    contract_api::{runtime, storage},
};
use casper_erc20::Address;
use casper_types::{
    bytesrepr::ToBytes, runtime_args, CLTyped, ContractPackageHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Key, Parameter, RuntimeArgs, U256,
};

const RESULT_KEY: &str = "result";
const TEST_CALL_KEY: &str = "test_call_package_hash";

fn store_result<T: CLTyped + ToBytes>(result: T) {
    match runtime::get_key(RESULT_KEY) {
        Some(Key::URef(uref)) => storage::write(uref, result),
        Some(_) => unreachable!(),
        None => {
            let new_uref = storage::new_uref(result);
            runtime::put_key(RESULT_KEY, new_uref.into());
        }
    }
}

#[no_mangle]
extern "C" fn get_max_flash_loan() {
    let token: Address = runtime::get_named_arg("token");
    let lender: ContractPackageHash = runtime::get_named_arg("lender");

    let maxflashloan: U256 = runtime::call_versioned_contract(
        lender,
        None,
        "max_flash_loan",
        runtime_args! {"token" => token},
    );
    store_result(maxflashloan);
}

#[no_mangle]
extern "C" fn get_flash_fee() {
    let token: Address = runtime::get_named_arg("token");
    let lender: ContractPackageHash = runtime::get_named_arg("lender");
    let amount: U256 = runtime::get_named_arg("amount");

    let flashfee: U256 = runtime::call_versioned_contract(
        lender,
        None,
        "flash_fee",
        runtime_args! {"token" => token,"amount" => amount},
    );
    store_result(flashfee);
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();
    let get_max_flash_loan_entrypoint = EntryPoint::new(
        String::from("get_max_flash_loan"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("lender", ContractPackageHash::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_flash_fee_entrypoint = EntryPoint::new(
        String::from("get_flash_fee"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("lender", ContractPackageHash::cl_type()),
            Parameter::new("amount", U256::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(get_max_flash_loan_entrypoint);
    entry_points.add_entry_point(get_flash_fee_entrypoint);

    let (_contract_hash, _version) =
        storage::new_contract(entry_points, None, Some(TEST_CALL_KEY.to_string()), None);
}
