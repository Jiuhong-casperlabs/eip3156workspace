use crate::constants::{
    ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, BALANCE_OF_ENTRY_POINT_NAME,
    CALLBACK_STRING, DATA_RUNTIME_ARG_NAME, FEE_RUNTIME_ARG_NAME, INITIATOR_RUNTIME_ARG_NAME,
    ON_FLASH_LOAN_ENTRY_POINT_NAME, OWNER_RUNTIME_ARG_NAME, RECIPIENT_RUNTIME_ARG_NAME,
    TOKEN_RUNTIME_ARG_NAME, TRANSFER_ENTRY_POINT_NAME, TRANSFER_FROM_ENTRY_POINT_NAME,
};
use crate::Error;
use casper_contract::contract_api::runtime;
use casper_erc20::Address;
use casper_types::{self, runtime_args, system::CallStackElement, RuntimeArgs, U256};

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

fn _flash_fee(fee: U256, amount: U256) -> Result<U256, Error> {
    // FlashLenderOverflow
    // amount * fee / 10000
    amount
        .checked_mul(fee)
        .ok_or(Error::FlashLenderOverflow)?
        .checked_div(U256::from(10000u128))
        .ok_or(Error::FlashLenderOverflow)
}

pub fn flash_loan(
    // flashlender: &EIP3156LENDER,
    fee: U256,
    receiver: Address,
    token: Address,
    amount: U256,
    data: String,
) -> Result<bool, Error> {
    let flashfee: U256 = _flash_fee(fee, amount).unwrap();

    // require(
    //     IERC20(token).transfer(address(receiver), amount),
    //     "FlashLender: Transfer failed"
    // );

    let token_package_hash = match token {
        Address::Contract(contractpackagehash) => contractpackagehash,
        _ => panic!("error"),
    };

    runtime::call_versioned_contract::<()>(
        token_package_hash,
        None,
        TRANSFER_ENTRY_POINT_NAME,
        runtime_args! {
            RECIPIENT_RUNTIME_ARG_NAME => receiver,
            AMOUNT_RUNTIME_ARG_NAME => amount,
        },
    );

    // require(
    //     receiver.onFlashLoan(msg.sender, token, amount, fee, data) == CALLBACK_SUCCESS,
    //     "FlashLender: Callback failed"
    // );

    let mut stacks = runtime::get_call_stack();
    let topstack = stacks.pop().unwrap(); // this contract => lender
    let previousstack = stacks.pop().unwrap(); // previous contract => msgsender/ loaner

    let msgsender = call_stack_element_to_address(previousstack);
    let callback_args = runtime_args! {
        INITIATOR_RUNTIME_ARG_NAME => msgsender,
        TOKEN_RUNTIME_ARG_NAME => token,
        AMOUNT_RUNTIME_ARG_NAME => amount,
        FEE_RUNTIME_ARG_NAME => flashfee,
        DATA_RUNTIME_ARG_NAME => data
    };

    let receiver_package_hash = match receiver {
        Address::Contract(contractpkhash) => contractpkhash,
        _ => return Err(Error::FlashLenderRepayFailed),
    };

    let bytes = CALLBACK_STRING.as_bytes();
    let callback_success: [u8; 32] = runtime::blake2b(bytes);

    let callback_result: [u8; 32] = runtime::call_versioned_contract(
        receiver_package_hash,
        None,
        ON_FLASH_LOAN_ENTRY_POINT_NAME,
        callback_args,
    );

    if callback_result != callback_success {
        return Err(Error::FlashLenderCallbackFailed);
    }
    // require(
    //     IERC20(token).transferFrom(address(receiver), address(this), amount + fee),
    //     "FlashLender: Repay failed"
    // );

    runtime::call_versioned_contract::<()>(
        token_package_hash,
        None,
        TRANSFER_FROM_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => receiver,
            RECIPIENT_RUNTIME_ARG_NAME => call_stack_element_to_address(topstack),
            AMOUNT_RUNTIME_ARG_NAME => amount + flashfee,
        },
    );

    Ok(true)
}

pub fn max_flash_loan(token: Address) -> U256 {
    let token_package_hash = match token {
        Address::Contract(contractpackagehash) => contractpackagehash,
        _ => panic!("error"),
    };

    let topstack = runtime::get_call_stack().pop().unwrap(); //this contract/lender
    runtime::call_versioned_contract::<U256>(
        token_package_hash,
        None,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            ADDRESS_RUNTIME_ARG_NAME => call_stack_element_to_address(topstack)
        },
    )
}

pub fn flash_fee(fee: U256, amount: U256) -> Result<U256, Error> {
    _flash_fee(fee, amount)
}
