//! Contains definition of the entry points.
use alloc::{string::String, vec};

use casper_types::{
    CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, U256,
};

use crate::constants::{
    AMOUNT_RUNTIME_ARG_NAME, DATA_RUNTIME_ARG_NAME, FLASH_FEE_ENTRY_POINT_NAME,
    FLASH_LOAN_ENTRY_POINT_NAME, MAX_FLASH_LOAN_ENTRY_POINT_NAME, RECEIVER_RUNTIME_ARG_NAME,
    TOKEN_RUNTIME_ARG_NAME,
};
use casper_erc20::Address;

/// Returns the `name` entry point.
pub fn flash_fee() -> EntryPoint {
    EntryPoint::new(
        String::from(FLASH_FEE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(TOKEN_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `symbol` entry point.
pub fn max_flash_loan() -> EntryPoint {
    EntryPoint::new(
        String::from(MAX_FLASH_LOAN_ENTRY_POINT_NAME),
        vec![Parameter::new(TOKEN_RUNTIME_ARG_NAME, Address::cl_type())],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
/// Returns the `transfer_from` entry point.
pub fn flash_loan() -> EntryPoint {
    EntryPoint::new(
        String::from(FLASH_LOAN_ENTRY_POINT_NAME),
        vec![
            Parameter::new(RECEIVER_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(TOKEN_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(DATA_RUNTIME_ARG_NAME, String::cl_type()),
        ],
        bool::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of EIP3156LENDER token entry points.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(max_flash_loan());
    entry_points.add_entry_point(flash_fee());
    entry_points.add_entry_point(flash_loan());

    entry_points
}
