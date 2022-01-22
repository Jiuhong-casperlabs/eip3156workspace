pub mod constants;
pub mod entry_points;
mod error;
mod flashlend;
mod tokens;
// mod eip3156borrower;
extern crate alloc;

use crate::constants::{LENDER_PACKAGE_NAME, LOAN_FEE_KEY_NAME, SUPPORT_TOKENS_KEY_NAME};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::Address;
use casper_types::{contracts::NamedKeys, EntryPoints, Key, URef, U256};
use once_cell::unsync::OnceCell;

pub use error::Error;
/// Implementation of ERC20 standard functionality.
#[derive(Default)]
pub struct EIP3156LENDER {
    suportted_tokens_uref: OnceCell<URef>,
    loan_fee_uref: OnceCell<URef>,
}

impl EIP3156LENDER {
    pub fn new(suportted_tokens_uref: URef, loan_fee_uref: URef) -> Self {
        Self {
            suportted_tokens_uref: suportted_tokens_uref.into(),
            loan_fee_uref: loan_fee_uref.into(),
            // loan_fee, matching loan fee is stored under dictionary
        }
    }

    fn suportted_tokens_uref(&self) -> URef {
        *self
            .suportted_tokens_uref
            .get_or_init(tokens::get_tokens_uref)
    }

    fn read_supported_tokens(&self) -> Vec<Address> {
        tokens::read_supported_tokens_from(self.suportted_tokens_uref())
    }

    fn loan_fee_uref(&self) -> URef {
        *self.loan_fee_uref.get_or_init(tokens::get_loanfee_uref)
    }

    fn read_loan_fee(&self, token: Address) -> U256 {
        tokens::read_loan_fee_from(self.loan_fee_uref(), token)
    }

    /// Returns the supported_tokens.
    pub fn supported_tokens(&self) -> Vec<Address> {
        self.read_supported_tokens()
    }

    pub fn flash_loan(
        &self,
        receiver: Address, //borrower
        token: Address,
        amount: U256,
        data: String,
    ) -> Result<bool, Error> {
        let supportted_tokens: Vec<Address> = self.read_supported_tokens();

        if supportted_tokens.contains(&token) {
            flashlend::flash_loan(self, receiver, token, amount, data)
        } else {
            Err(Error::FlashLenderUnsupportedCurrency)
        }
    }

    pub fn flash_fee(&self, token: Address, amount: U256) -> Result<U256, Error> {
        let supportted_tokens: Vec<Address> = self.read_supported_tokens();

        if supportted_tokens.contains(&token) {
            Ok(flashlend::flash_fee(self, token, amount))
        } else {
            Err(Error::FlashLenderUnsupportedCurrency)
        }
    }

    pub fn max_flash_loan(&self, token: Address) -> U256 {
        let supportted_tokens: Vec<Address> = self.read_supported_tokens();

        if supportted_tokens.contains(&token) {
            flashlend::max_flash_loan(token)
        } else {
            U256::from(0)
        }
    }

    /// This should be called from within `fn call()` of your contract.
    pub fn install(initial_supported_tokens: Vec<(Address, U256)>) -> Result<EIP3156LENDER, Error> {
        let default_entry_points = entry_points::default();
        EIP3156LENDER::install_custom(initial_supported_tokens, default_entry_points)
    }

    /// Installs the ERC20 contract with a custom set of entry points.
    ///
    /// # Warning
    ///
    /// Contract developers should use [`ERC20::install`] instead, as it will create the default set
    /// of ERC20 entry points. Using `install_custom` with a different set of entry points might
    /// lead to problems with integrators such as wallets, and exchanges.
    #[doc(hidden)]
    pub fn install_custom(
        initial_supported_tokens: Vec<(Address, U256)>,
        entry_points: EntryPoints,
    ) -> Result<EIP3156LENDER, Error> {
        let loan_fee_uref = storage::new_dictionary(LOAN_FEE_KEY_NAME).unwrap_or_revert();
        let loan_fee_key = Key::from(loan_fee_uref);

        let mut supported_tokens: Vec<Address> = vec![];
        for (_, n) in initial_supported_tokens.into_iter().enumerate() {
            supported_tokens.push(n.0); //fill vec of tokens address
            tokens::write_loan_fee_to(loan_fee_uref, n.0, n.1) //fill dictionary with loan fee and token pair
        }

        let supported_tokens_uref = storage::new_uref(supported_tokens).into_read_write();
        let supported_tokens_key = Key::from(supported_tokens_uref);

        let mut named_keys = NamedKeys::new();

        named_keys.insert(SUPPORT_TOKENS_KEY_NAME.to_string(), supported_tokens_key);
        named_keys.insert(LOAN_FEE_KEY_NAME.to_string(), loan_fee_key);

        let (contract_package_hash, _) = storage::create_contract_package_at_hash();

        storage::add_contract_version(contract_package_hash, entry_points, named_keys);

        runtime::remove_key(LOAN_FEE_KEY_NAME);
        runtime::put_key(LENDER_PACKAGE_NAME, contract_package_hash.into());

        Ok(EIP3156LENDER::new(supported_tokens_uref, loan_fee_uref))
    }
}
