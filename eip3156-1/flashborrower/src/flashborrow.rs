use crate::constants::{
    ALLOWANCE_ENTRY_POINT_NAME, AMOUNT_RUNTIME_ARG_NAME, APPROVE_ENTRY_POINT_NAME, CALLBACK_STRING,
    DATA_RUNTIME_ARG_NAME, FLASH_FEE_ENTRY_POINT_NAME, FLASH_LOAN_ENTRY_POINT_NAME,
    OWNER_RUNTIME_ARG_NAME, RECEIVER_RUNTIME_ARG_NAME, SPENDER_RUNTIME_ARG_NAME,
    TOKEN_RUNTIME_ARG_NAME,
};
use crate::Error;
use casper_contract::contract_api::runtime;
use casper_erc20::Address;
use casper_types::{runtime_args, system::CallStackElement, RuntimeArgs, U256};

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

pub fn on_flash_loan(
    lender: Address,
    initiator: Address,
    _token: Address,
    _amount: U256,
    _fee: U256,
    _data: String,
) -> Result<[u8; 32], Error> {
    let mut stacks = runtime::get_call_stack();
    // currect stack
    let topstack = stacks.pop().unwrap();

    // lender contract / previous stack
    let previousstack = stacks.pop().unwrap();

    // require(
    //     msg.sender == address(lender),
    //     "FlashBorrower: Untrusted lender"
    // );

    if call_stack_element_to_address(previousstack) != lender {
        return Err(Error::FlashBorrowerUntrustedFender);
    }
    // require(
    //     initiator == address(this),
    //     "FlashBorrower: Untrusted loan initiator"
    // );
    if initiator != call_stack_element_to_address(topstack) {
        return Err(Error::FlashBorrowerUntrustedLoanInitiator);
    }

    // TO DO: Action about data -> plan to add history
    // TO DO: Action about data -> plan to add history
    // TO DO: Action about data -> plan to add history

    let bytes = CALLBACK_STRING.as_bytes();

    let callback_success: [u8; 32] = runtime::blake2b(bytes);

    Ok(callback_success)
}

pub fn flash_borrow(lender: Address, token: Address, amount: U256) -> Result<(), Error> {
    // allowance start
    let erc20_package_hash = match token {
        Address::Contract(contractpkhash) => contractpkhash,
        _ => panic!("incorrect token"),
    };

    let mut stacks = runtime::get_call_stack();
    // current stack
    let topstack = stacks.pop().unwrap();

    let topaddress = call_stack_element_to_address(topstack);

    let _allowance: U256 = runtime::call_versioned_contract(
        erc20_package_hash,
        None,
        ALLOWANCE_ENTRY_POINT_NAME,
        runtime_args! {
            OWNER_RUNTIME_ARG_NAME => topaddress,
            SPENDER_RUNTIME_ARG_NAME => lender
        },
    );
    // allowance end

    // invoke entrypoint flashfee of lender contract start
    let lender_package_hash = match lender {
        Address::Contract(contractpkhash) => contractpkhash,
        _ => panic!("error"),
    };

    let flashfee: U256 = runtime::call_versioned_contract(
        lender_package_hash,
        None,
        FLASH_FEE_ENTRY_POINT_NAME,
        runtime_args! {
            TOKEN_RUNTIME_ARG_NAME=> token,
            AMOUNT_RUNTIME_ARG_NAME=> amount,
        },
    );

    // flash fee end

    // repayment
    let _repayment: U256 = amount + flashfee;

    // IERC20(token).approve(address(lender), _allowance + _repayment);

    // let approve_result: Result<(), Error> = runtime::call_versioned_contract(
    runtime::call_versioned_contract::<()>(
        erc20_package_hash,
        None,
        APPROVE_ENTRY_POINT_NAME,
        runtime_args! {
            SPENDER_RUNTIME_ARG_NAME => lender,
            AMOUNT_RUNTIME_ARG_NAME => _allowance + _repayment,
        },
    );

    // lender.flashLoan(this, token, amount, data);
    let result: bool = runtime::call_versioned_contract(
        lender_package_hash,
        None,
        FLASH_LOAN_ENTRY_POINT_NAME,
        runtime_args! {
            RECEIVER_RUNTIME_ARG_NAME=> topaddress,
            TOKEN_RUNTIME_ARG_NAME => token,
            AMOUNT_RUNTIME_ARG_NAME=> amount,
            DATA_RUNTIME_ARG_NAME => "data",
        },
    );

    if !result {
        return Err(Error::FlashBorrowerInitialFailed);
    }
    Ok(())
}
