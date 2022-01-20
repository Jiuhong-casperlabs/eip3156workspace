mod error;
extern crate alloc;
mod constants;
mod detail;
pub mod entry_points;
mod flashborrow;
use crate::constants::LENDER_KEY_NAME;

use casper_contract::contract_api::{runtime, storage};
use casper_erc20::Address;
use casper_types::{contracts::NamedKeys, EntryPoints, Key, URef, U256};
use once_cell::unsync::OnceCell;

pub use error::Error;
/// Implementation of ERC20 standard functionality.
#[derive(Default)]
pub struct EIP3156BORROWER {
    lender: OnceCell<URef>,
}

impl EIP3156BORROWER {
    pub fn new(lender_uref: URef) -> Self {
        Self {
            lender: lender_uref.into(),
        }
    }

    fn lender_uref(&self) -> URef {
        *self.lender.get_or_init(detail::get_lender_uref)
    }

    fn read_lender_address(&self) -> Address {
        // storage::read(self.lender).unwrap_or_revert().unwrap_or_revert()
        detail::read_lender_address_from(self.lender_uref())
    }

    pub fn on_flash_loan(
        &self,
        initiator: Address,
        token: Address,
        amount: U256,
        fee: U256,
        data: String,
    ) -> Result<[u8; 32], Error> {
        // Read lender address
        let lender = self.read_lender_address();
        flashborrow::on_flash_loan(lender, initiator, token, amount, fee, data)
    }

    // plan to add history to this contract by dictionary
    // (Action action) = abi.decode(data, (Action));
    // if (action == Action.NORMAL) {
    //     // do one thing
    // } else if (action == Action.OTHER) {
    //     // do another
    // }
    // return keccak256("ERC3156FlashBorrower.onFlashLoan");

    /// @dev Initiate a flash loan
    pub fn flash_borrow(&self, token: Address, amount: U256) -> Result<(), Error> {
        let lender = self.read_lender_address();

        flashborrow::flash_borrow(lender, token, amount)
        // here here here here
        // IERC20(token).approve(address(lender), _allowance + _repayment);
        // lender.flashLoan(this, token, amount, data);
    }

    /// This should be called from within `fn call()` of your contract.
    pub fn install(lender_address: Address) -> Result<EIP3156BORROWER, Error> {
        let default_entry_points = entry_points::default();
        EIP3156BORROWER::install_custom(lender_address, default_entry_points)
    }

    pub fn install_custom(
        lender_address: Address,
        entry_points: EntryPoints,
    ) -> Result<EIP3156BORROWER, Error> {
        let lender_uref = storage::new_uref(lender_address);
        let lender_key = Key::from(lender_uref);

        let mut named_keys = NamedKeys::new();
        named_keys.insert(LENDER_KEY_NAME.to_string(), lender_key);

        let (contract_hash, _version) =
            storage::new_locked_contract(entry_points, Some(named_keys), None, None);

        runtime::put_key("BORROWER", Key::from(contract_hash));
        runtime::remove_key(LENDER_KEY_NAME);
        Ok(EIP3156BORROWER::new(lender_uref))
    }
}
