use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::Address;
use casper_types::{ApiError, URef};

/// Reads a total supply from a specified [`URef`].
pub(crate) fn read_lender_address_from(uref: URef) -> Address {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

fn get_uref(name: &str) -> URef {
    let key = runtime::get_key(name)
        .ok_or(ApiError::MissingKey)
        .unwrap_or_revert();
    std::convert::TryInto::try_into(key).unwrap_or_revert()
}

pub fn get_lender_uref() -> URef {
    get_uref("lender")
}
