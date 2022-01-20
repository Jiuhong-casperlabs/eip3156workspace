//! Implementation details.
use casper_contract::contract_api::runtime;
use casper_types::{
    bytesrepr::Bytes, runtime_args, system::CallStackElement, ContractPackageHash, RuntimeArgs,
    U256,
};

use crate::{Address, ERC20};
mod error_flashmint;
pub use error_flashmint::Error;

/// Returns address based on a [`CallStackElement`].
///
/// For `Session` and `StoredSession` variants it will return account hash, and for `StoredContract`
/// case it will use contract hash as the address.

fn address_to_package_hash(address: Address) -> ContractPackageHash {
    match address {
        Address::Contract(contractpkhash) => contractpkhash,
        _ => panic!("incorrect"),
    }
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

/// Gets the immediate call stack element of the current execution.
fn get_top_call_stack_item() -> CallStackElement {
    let call_stack = runtime::get_call_stack();
    call_stack.into_iter().rev().next().unwrap()
}

fn get_second_call_stack_item() -> CallStackElement {
    let call_stack = runtime::get_call_stack();
    call_stack.into_iter().rev().nth(1).unwrap()
}

fn get_top_caller_address() -> Address {
    call_stack_element_to_address(get_top_call_stack_item())
}

fn get_second_caller_address() -> Address {
    call_stack_element_to_address(get_second_call_stack_item())
}

fn _flash_fee(amount: U256, fee: U256) -> U256 {
    amount * fee / 10000
}

pub(crate) fn flash_fee(erc20: &mut ERC20, token: Address, amount: U256) -> Result<U256, Error> {
    if token != get_top_caller_address() {
        Err(Error::FlashLenderUnsupportedCurrency)
    } else {
        Ok(_flash_fee(amount, erc20.loan_fee()))
    }
}

pub(crate) fn flash_loan(
    erc20: &mut ERC20,
    receiver: Address,
    token: Address,
    amount: U256,
    data: Bytes,
) -> Result<bool, Error> {
    if token != get_top_caller_address() {
        return Err(Error::FlashLenderUnsupportedCurrency);
    }

    let flashfee = match erc20.flash_fee(token, amount) {
        Ok(a) => a,
        Err(_) => panic!("123"),
    };

    let result = erc20.mint(receiver, amount);
    if result.is_err() {
        panic!("3456")
    }
    // require(
    //     receiver.onFlashLoan(msg.sender, token, amount, fee, data) == CALLBACK_SUCCESS,
    //     "FlashMinter: Callback failed"
    // );
    let receiver_package_hash = address_to_package_hash(receiver);

    let msgsender = get_second_caller_address();
    let callback_args = runtime_args! {
        "sender" => msgsender,
        "token" => token,
        "amount" => amount,
        "fee" => flashfee,
        "data" => data
    };

    let callback_result: Result<[u8; 32], Error> =
        runtime::call_versioned_contract(receiver_package_hash, None, "onFlashLoan", callback_args);

    let string = "ERC3156FlashBorrower.onFlashLoan";
    let bytes = string.as_bytes();
    let callback_success: [u8; 32] = runtime::blake2b(bytes);

    let callback_hash = match callback_result {
        Ok(hash) => hash,
        Err(_) => return Err(Error::FlashMinterCallbackFailed),
    };

    if callback_hash != callback_success {
        return Err(Error::FlashMinterCallbackFailed);
    }

    // allowance
    let _allowance: U256 = erc20.allowance(receiver, get_top_caller_address());
    if _allowance < amount + flashfee {
        return Err(Error::FlashMinterCallbackFailed);
    }

    let result = erc20.approve(get_top_caller_address(), _allowance - (amount + flashfee));

    if result.is_err() {
        panic!("123")
    }

    let result = erc20.burn(receiver, amount + flashfee);
    if result.is_err() {
        panic!("234")
    }

    Ok(true)
}
