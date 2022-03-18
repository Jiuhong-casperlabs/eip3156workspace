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
use casper_erc20::{
    constants::{
        AMOUNT_RUNTIME_ARG_NAME, APPROVE_ENTRY_POINT_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        TRANSFER_ENTRY_POINT_NAME, TRANSFER_FROM_ENTRY_POINT_NAME,
    },
    Address,
};
use casper_types::{
    bytesrepr::ToBytes, runtime_args, system::CallStackElement, CLTyped, ContractHash,
    ContractPackageHash, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter,
    RuntimeArgs, U256,
};

const CHECK_TOTAL_SUPPLY_ENTRY_POINT_NAME: &str = "check_total_supply";
const CHECK_BALANCE_OF_ENTRY_POINT_NAME: &str = "check_balance_of";
const TRANSFER_AS_STORED_CONTRACT_ENTRY_POINT_NAME: &str = "transfer_as_stored_contract";
const APPROVE_AS_STORED_CONTRACT_ENTRY_POINT_NAME: &str = "approve_as_stored_contract";
const TRANSFER_FROM_AS_STORED_CONTRACT_ENTRY_POINT_NAME: &str = "transfer_from_as_stored_contract";
const CHECK_ALLOWANCE_OF_ENTRY_POINT_NAME: &str = "check_allowance_of";
const TOKEN_CONTRACT_RUNTIME_ARG_NAME: &str = "token_contract";
const ADDRESS_RUNTIME_ARG_NAME: &str = "address";
const OWNER_RUNTIME_ARG_NAME: &str = "owner";
const SPENDER_RUNTIME_ARG_NAME: &str = "spender";
const RESULT_KEY: &str = "result";
const TEST_CALL_KEY: &str = "minter_test_call_package_hash";

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

fn call_stack_element_to_address(call_stack_element: CallStackElement) -> Key {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Key::from(account_hash),
        CallStackElement::StoredSession {
            contract_package_hash,
            ..
        } => Key::from(contract_package_hash),
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Key::from(contract_package_hash),
    }
}

fn get_top_call_stack_item() -> CallStackElement {
    let call_stack = runtime::get_call_stack();
    call_stack.into_iter().rev().next().unwrap()
}

fn get_top_caller_address() -> Key {
    call_stack_element_to_address(get_top_call_stack_item())
}

#[no_mangle]
extern "C" fn transfer() {
    let erc20_str: String = runtime::get_named_arg("erc20_str");
    // contract-d89f0cf675955270b8ad64eb2b412f5c16e03e28ff71724276ddb7785f7045e2
    let erc20 = ContractHash::from_formatted_str(&erc20_str).unwrap();
    let amount: U256 = runtime::get_named_arg("amount");
    let recipient = get_top_caller_address();

    let result: bool = runtime::call_contract(
        erc20,
        "transfer",
        runtime_args! {
         "recipient" =>recipient,
         "amount" => amount,
        },
    );
}

#[no_mangle]
extern "C" fn get_flash_loan() {
    let token: Key = runtime::get_named_arg("token");

    let hash = token.into_hash().unwrap();
    let contractpackagehash = ContractPackageHash::new(hash);
    let amount: U256 = runtime::get_named_arg("amount");
    let data: String = runtime::get_named_arg("data");
    let receiver = get_top_caller_address();

    let result: bool = runtime::call_versioned_contract(
        contractpackagehash,
        None,
        "flash_loan",
        runtime_args! {
         "receiver" =>receiver,
         "token" => token,
         "amount" => amount,
        "data"=> data},
    );
    store_result(result);
}

#[no_mangle]
extern "C" fn get_max_flash_loan() {
    let minter_str: String = runtime::get_named_arg("minter_str");
    let minter = ContractHash::from_formatted_str(&minter_str).unwrap();

    let maxflashloan: U256 = runtime::call_contract(minter, "max_flash_loan", runtime_args! {});
    store_result(maxflashloan);
}

#[no_mangle]
extern "C" fn get_loan_fee() {
    let minter_str: String = runtime::get_named_arg("minter_str");
    let minter = ContractHash::from_formatted_str(&minter_str).unwrap();

    let loanfee: U256 = runtime::call_contract(minter, "loan_fee", runtime_args! {});
    store_result(loanfee);
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();

    let transfer_entrypoint = EntryPoint::new(
        String::from("transfer"),
        vec![
            Parameter::new("recipient", Key::cl_type()),
            Parameter::new("amount", U256::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_flash_loan_entrypoint = EntryPoint::new(
        String::from("get_flash_loan"),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("amount", U256::cl_type()),
            Parameter::new("data", String::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_max_flash_loan_entrypoint = EntryPoint::new(
        String::from("get_max_flash_loan"),
        vec![Parameter::new("minter_str", String::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let get_loan_fee_entrypoint = EntryPoint::new(
        String::from("get_loan_fee"),
        vec![Parameter::new("minter_str", String::cl_type())],
        <()>::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entry_points.add_entry_point(transfer_entrypoint);
    entry_points.add_entry_point(get_flash_loan_entrypoint);
    entry_points.add_entry_point(get_max_flash_loan_entrypoint);
    entry_points.add_entry_point(get_loan_fee_entrypoint);

    let (_contract_hash, _version) =
        storage::new_contract(entry_points, None, Some(TEST_CALL_KEY.to_string()), None);
}
