use crate::{Error, EIP3156LENDER};
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

fn _flash_fee(flashlender: &EIP3156LENDER, token: Address, amount: U256) -> U256 {
    let fee = flashlender.read_loan_fee(token);
    amount * fee / 10000
}

pub fn flash_loan(
    flashlender: &EIP3156LENDER,
    receiver: Address,
    token: Address,
    amount: U256,
    data: String,
) -> Result<bool, Error> {
    let flashfee: U256 = _flash_fee(flashlender, token, amount);

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
        "transfer",
        runtime_args! {
            "recipient" => receiver,
            "amount" => amount,
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
        "initiator" => msgsender,
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

    let callback_result: [u8; 32] = runtime::call_versioned_contract(
        receiver_package_hash,
        None,
        "on_flash_loan",
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
        "transfer_from",
        runtime_args! {
            "owner" => receiver,
            "recipient" => call_stack_element_to_address(topstack),
            "amount" => amount + flashfee,
        },
    );

    // if result_transfer_from.is_err() {
    //     return Err(Error::FlashLenderRepayFailed);
    // }

    Ok(true)
}

pub fn max_flash_loan(token: Address) -> U256 {
    let token_package_hash = match token {
        Address::Contract(contractpackagehash) => contractpackagehash,
        _ => panic!("error"),
    };
    // let token_name:String = runtime::call_versioned_contract(contract_package_hash, None, "name", runtime_args!{});

    let topstack = runtime::get_call_stack().pop().unwrap(); //this contract/lender
    runtime::call_versioned_contract::<U256>(
        token_package_hash,
        None,
        "balance_of",
        runtime_args! {
            "address" => call_stack_element_to_address(topstack)
        },
    )
}

pub fn flash_fee(flashlender: &EIP3156LENDER, token: Address, amount: U256) -> U256 {
    _flash_fee(flashlender, token, amount)
}
