mod error;
use casper_contract::contract_api::{self, runtime};
use casper_erc20::{entry_points::symbol, Address};
use casper_types::{
    self, bytesrepr::Bytes, runtime_args, system::CallStackElement, ContractPackageHash,
    RuntimeArgs, U256,
};

pub use error::Error;

pub struct EIP3156MINTER {
    suportted_tokens: Vec<Address>,
    fee: U256,
}

fn call_stack_element_to_address(call_stack_element: CallStackElement) -> Address {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Address::from(account_hash),
        CallStackElement::StoredSession { account_hash, .. } => {
            // Stored session code acts in account's context, so if stored session wants to interact
            // with an ERC20 token caller's address will be used.
            Address::from(account_hash)
        }
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Address::from(contract_package_hash),
    }
}

impl EIP3156MINTER {
    pub fn new(suportted_tokens: Vec<Address>, fee: U256) -> Self {
        Self {
            suportted_tokens,
            fee,
        }
    }

    fn get_contract_package_hash(&self, token: Address) -> Result<ContractPackageHash, Error> {
        match token {
            Address::Contract(contractpackagehash) => return Ok(contractpackagehash),
            _ => return Err(Error::IncorrectAddress),
        };
    }
    pub fn max_flash_loan(&self, token: Address) -> U256 {
        if let Ok(token_package_hash) = self.get_contract_package_hash(token) {
            let totalsupply: U256 = runtime::call_versioned_contract(
                token_package_hash,
                None,
                "total_supply",
                runtime_args! {},
            );
            U256::MAX - totalsupply
        } else {
            U256::from(0)
        }
    }

    fn _flash_fee(&self, _token: Address, amount: U256) -> U256 {
        amount * self.fee / 10000
    }

    pub fn flash_fee(&self, token: Address, amount: U256) -> Result<U256, Error> {
        if !self.suportted_tokens.contains(&token) {
            // TODO:
            // return error FlashLender: Unsupported currency
            Err(Error::FlashMinterUnsupportedCurrency)
        } else {
            Ok(self._flash_fee(token, amount))
        }
    }

    pub fn flash_loan(
        &self,
        receiver: Address,
        token: Address,
        amount: U256,
        data: Bytes,
    ) -> Result<bool, Error> {
        if !self.suportted_tokens.contains(&token) {
            // TODO:
            // return error FlashMinter: Unsupported currency
            return Err(Error::FlashMinterUnsupportedCurrency);
        }
        let flashfee: U256 = self._flash_fee(token, amount);

        // require(
        //     IERC20(token).transfer(address(receiver), amount),
        //     "FlashLender: Transfer failed"
        // );

        // let token_package_hash = match token {
        //     Address::Contract(contractpackagehash) => contractpackagehash,
        //     _ => panic!("error"),
        // };

        if let Ok(token_package_hash) = self.get_contract_package_hash(token) {
            let totalsupply: U256 = runtime::call_versioned_contract(
                token_package_hash,
                None,
                "mint",
                runtime_args! {},
            );
            U256::MAX - totalsupply
        } else {
            return Err(Error::FlashBorrowerUntrustedFender);
        }

        let transfer_result: Result<(), Error> = runtime::call_versioned_contract(
            token_package_hash,
            None,
            "transfer",
            runtime_args! {
                "recipient" => receiver,
                "amount" => amount,
            },
        );

        if transfer_result.is_err() {
            return Err(Error::FlashLenderTransferFailed);
        }

        // require(
        //     receiver.onFlashLoan(msg.sender, token, amount, fee, data) == CALLBACK_SUCCESS,
        //     "FlashLender: Callback failed"
        // );

        let mut stacks = runtime::get_call_stack();
        let topstack = stacks.pop().unwrap(); // this contract => lender
        let secondstack = stacks.pop().unwrap(); // previous contract => msgsender/ loaner

        let msgsender = call_stack_element_to_address(secondstack);
        let callback_args = runtime_args! {
            "sender" => msgsender,
            "token" => token,
            "amount" => amount,
            "fee" => flashfee,
            "data" => data
        };

        let receiver_package_hash = match receiver {
            Address::Contract(contractpkhash) => contractpkhash,
            _ => return Err(Error::FlashLenderRepayFailed),
        };

        let string = "ERC3156FlashBorrower.onFlashLoan";
        let bytes = string.as_bytes();
        let callback_success: [u8; 32] = runtime::blake2b(bytes);

        let callback_result: Result<[u8; 32], Error> = runtime::call_versioned_contract(
            receiver_package_hash,
            None,
            "onFlashLoan",
            callback_args,
        );

        let callback_hash = match callback_result {
            Ok(hash) => hash,
            Err(_) => return Err(Error::FlashLenderCallbackFailed),
        };

        if callback_hash != callback_success {
            return Err(Error::FlashLenderCallbackFailed);
        }
        // require(
        //     IERC20(token).transferFrom(address(receiver), address(this), amount + fee),
        //     "FlashLender: Repay failed"
        // );

        let result_transfer_from: Result<(), Error> = runtime::call_versioned_contract(
            token_package_hash,
            None,
            "transfer_from",
            runtime_args! {
                "owner" => receiver,
                "recipient" => call_stack_element_to_address(topstack),
                "amount" => amount + flashfee,
            },
        );

        if result_transfer_from.is_err() {
            return Err(Error::FlashLenderRepayFailed);
        }

        Ok(true)
    }
}
