//! Contains definition of the entry points.
use alloc::{string::String, vec};

use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, U256,
};

use crate::constants::{
    AMOUNT_RUNTIME_ARG_NAME, DATA_RUNTIME_ARG_NAME, FEE_RUNTIME_ARG_NAME,
    FLASH_BORROW_ENTRY_POINT_NAME, INITIATOR_RUNTIME_ARG_NAME, ON_FLASH_LOAN_ENTRY_POINT_NAME,
    TOKEN_RUNTIME_ARG_NAME,
};
use casper_erc20::Address;

/// Returns the `on_flash_loan` entry point.
pub fn on_flash_loan() -> EntryPoint {
    EntryPoint::new(
        String::from(ON_FLASH_LOAN_ENTRY_POINT_NAME),
        vec![
            Parameter::new(INITIATOR_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(TOKEN_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(FEE_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(DATA_RUNTIME_ARG_NAME, String::cl_type()),
        ],
        CLType::ByteArray(32),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `flash_borrow` entry point.
pub fn flash_borrow() -> EntryPoint {
    EntryPoint::new(
        String::from(FLASH_BORROW_ENTRY_POINT_NAME),
        vec![
            Parameter::new(TOKEN_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of EIP3156LENDER token entry points.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(on_flash_loan());
    entry_points.add_entry_point(flash_borrow());

    entry_points
}
