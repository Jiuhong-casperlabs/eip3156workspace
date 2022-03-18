#[cfg(test)]
mod tests {
    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
        DEFAULT_PAYMENT,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };

    use casper_execution_engine::core::{
        engine_state::{Error as CoreError, ExecuteRequest},
        execution::Error as ExecError,
    };
    use casper_types::{
        account::AccountHash,
        bytesrepr::{FromBytes, ToBytes},
        runtime_args, ApiError, CLTyped, ContractPackageHash, Key, Motes, PublicKey, RuntimeArgs,
        SecretKey, U256, U512,
    };

    const ERC20_WASM: &str =
        "/home/jh/mywork/eip3156workspace/eip3156-1/target/wasm32-unknown-unknown/release/minter.wasm";
    const LENDER_WASM: &str = "lender.wasm";
    const BORROWER_WASM: &str = "borrower.wasm";
    const TEST_CALL_WASM: &str = "eip3156_test_call.wasm";
    const ARG_INITIAL_SUPPORTED_TOKENS: &str = "initial_supported_tokens";
    const LENDER_ADDRESS: &str = "lender_address";
    const LENDER_PACKAGE_HASH_KEY: &str = "LENDER";
    const BORROWER_PACKAGE_HASH_KEY: &str = "BORROWER";
    const ERC20_CONTRACT_KEY: &str = "erc20_token_contract";
    const ERC20_PACKAGE_KEY: &str = "erc20_package_hash";
    const ERROR_INSUFFICIENT_BALANCE: u16 = u16::MAX - 1;
    const ERROR_FFLASH_LENDER_OVERFLOW: u16 = u16::MAX - 210;

    struct TestFixture {
        account_address: AccountHash,
        test_call_package_hash_key: Key,
        erc20_package_hash_key: Key,
        lender_package_hash_key: Key,
        borrower_package_hash_key: Key,
    }

    fn get_test_result<T: FromBytes + CLTyped>(
        builder: &mut InMemoryWasmTestBuilder,
        test_contract_package_hash: ContractPackageHash,
    ) -> T {
        let contract_package = builder
            .get_contract_package(test_contract_package_hash)
            .expect("should have contract package");
        let enabled_versions = contract_package.enabled_versions();
        let (_version, contract_hash) = enabled_versions
            .iter()
            .rev()
            .next()
            .expect("should have latest version");

        builder.get_value(*contract_hash, "result")
    }

    fn setup() -> (InMemoryWasmTestBuilder, TestFixture) {
        // Create an asymmetric keypair, and derive the account address of this.
        let secret_key = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_address = AccountHash::from(&public_key);

        // Make this account a genesis account (one which exists at network startup) and a
        // genesis request for the execution engine.
        let account = GenesisAccount::account(
            public_key,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );

        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);

        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );

        let mut test_builder = InMemoryWasmTestBuilder::default();
        test_builder.run_genesis(&run_genesis_request).commit();

        // ====install erc20 contract start=========//
        let exec_request = {
            ExecuteRequestBuilder::standard(
                account_address,
                ERC20_WASM,
                runtime_args! {
                "name"=> String::from("ORANGE"),
                "symbol" => String::from("OOO"),
                "decimals" => 10u8,
                "fee" => U256::from(10),
                "total_supply" => U256::from(1000000000000000u128)
                },
            )
            .build()
        };

        test_builder.exec(exec_request).expect_success().commit();

        // ======install erc20 contract end =========//

        //get account
        let account = test_builder
            .query(None, Key::Account(account_address), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        // ========= install lender contract start========= //

        //get erc20 package hash
        let erc20_package_hash_key = *account
            .named_keys()
            .get(ERC20_PACKAGE_KEY)
            .expect("should have erc20 contract");

        let exec_request = {
            ExecuteRequestBuilder::standard(
                account_address,
                LENDER_WASM,
                runtime_args! {
                    ARG_INITIAL_SUPPORTED_TOKENS => vec![(erc20_package_hash_key, U256::from(10))]
                },
            )
            .build()
        };

        test_builder.exec(exec_request).expect_success().commit();

        // ========= install lender contract end========= //

        // get account
        let account = test_builder
            .query(None, Key::Account(account_address), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        //get lender package hash
        let lender_package_hash = account
            .named_keys()
            .get(LENDER_PACKAGE_HASH_KEY)
            .expect("should have lender package");

        // ========= install borrower contract start========= //
        let exec_request = {
            ExecuteRequestBuilder::standard(
                account_address,
                BORROWER_WASM,
                runtime_args! {
                    LENDER_ADDRESS => *lender_package_hash
                },
            )
            .build()
        };

        test_builder.exec(exec_request).expect_success().commit();

        // ========= install borrower contract end========= //

        // get account
        let account = test_builder
            .query(None, Key::Account(account_address), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        let lender_package_hash_key = *(account
            .named_keys()
            .get(LENDER_PACKAGE_HASH_KEY)
            .expect("should have lender package hash"));

        let borrower_package_hash_key = *(account
            .named_keys()
            .get(BORROWER_PACKAGE_HASH_KEY)
            .expect("should have borrower package hash"));

        // =======install test-call start ============
        //get erc20 package hash
        let erc20_package_hash_key = *account
            .named_keys()
            .get(ERC20_PACKAGE_KEY)
            .expect("should have erc20 contract");

        //  install test-call
        let exec_request = {
            ExecuteRequestBuilder::standard(account_address, TEST_CALL_WASM, runtime_args! {})
                .build()
        };

        test_builder.exec(exec_request).expect_success().commit();
        // =======install test-call end   ============
        // get account
        let account = test_builder
            .query(None, Key::Account(account_address), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        let test_call_package_hash_key = *(account
            .named_keys()
            .get("test_call_package_hash")
            .expect("should have test_call_package hash"));

        let test_context = TestFixture {
            account_address,
            test_call_package_hash_key,
            erc20_package_hash_key,
            lender_package_hash_key,
            borrower_package_hash_key,
        };
        (test_builder, test_context)
    }

    fn tranfer_erc20(
        builder: &mut InMemoryWasmTestBuilder,
        test_context: &TestFixture,
        to_lender: bool,
        amount: U256,
    ) {
        let deploy = DeployItemBuilder::new()
            .with_address(test_context.account_address)
            .with_stored_session_named_key(
                "erc20_token_contract",
                "transfer",
                runtime_args! {
                    "recipient" =>  if to_lender {test_context.lender_package_hash_key} else {test_context.borrower_package_hash_key},
                    "amount" => amount
                },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[test_context.account_address])
            .with_deploy_hash([42; 32])
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy).build();
        builder.exec(execute_request).commit().expect_success();
    }

    fn make_flash_borrow_request(test_context: &TestFixture, amount: U256) -> ExecuteRequest {
        // get borrower package hash
        let borrower_package_hash = test_context
            .borrower_package_hash_key
            .into_hash()
            .map(ContractPackageHash::new)
            .expect("should be hash");

        let deploy = DeployItemBuilder::new()
            .with_address(test_context.account_address)
            .with_stored_versioned_contract_by_hash(
                borrower_package_hash.value(),
                None,
                "flash_borrow",
                runtime_args! {
                    "token" => test_context.erc20_package_hash_key,
                    "amount" => amount,
                },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[test_context.account_address])
            .with_deploy_hash([42; 32])
            .build();
        ExecuteRequestBuilder::from_deploy_item(deploy).build()
    }

    fn balance_of(
        builder: &mut InMemoryWasmTestBuilder,
        test_context: &TestFixture,
        lender: bool,
    ) -> U256 {
        //get balance_uref
        let balance_uref = *builder
            .query(
                None,
                Key::Account(test_context.account_address),
                &[ERC20_CONTRACT_KEY.to_string()],
            )
            .expect("should have validator slots")
            .as_contract()
            .expect("should be contractpackage")
            .clone()
            .take_named_keys()
            .get("balances")
            .unwrap()
            .clone()
            .as_uref()
            .unwrap();

        let dic_item_key = base64::encode(
            if lender {
                &test_context.lender_package_hash_key
            } else {
                &test_context.borrower_package_hash_key
            }
            .to_bytes()
            .unwrap(),
        );

        builder
            .query_dictionary_item(None, balance_uref, &dic_item_key)
            .ok()
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t()
            .unwrap()
        // println!("herherhereh is {}", value);
    }

    fn max_flash_loan(builder: &mut InMemoryWasmTestBuilder, test_context: &TestFixture) -> U256 {
        let test_call_contract_package = test_context
            .test_call_package_hash_key
            .into_hash()
            .map(ContractPackageHash::new)
            .expect("should be package hash");

        let lender_contract_package = test_context
            .lender_package_hash_key
            .into_hash()
            .map(ContractPackageHash::new)
            .expect("should be hash");
        let deploy = DeployItemBuilder::new()
            .with_address(test_context.account_address)
            .with_stored_versioned_contract_by_name(
                "test_call_package_hash",
                None,
                "get_max_flash_loan",
                runtime_args! {
                    "token" => test_context.erc20_package_hash_key,
                    "lender" => lender_contract_package
                },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[test_context.account_address])
            .with_deploy_hash([42; 32])
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy).build();
        builder.exec(execute_request).commit().expect_success();

        get_test_result(builder, test_call_contract_package)
    }

    fn flash_fee(
        builder: &mut InMemoryWasmTestBuilder,
        test_context: &TestFixture,
        amount: U256,
    ) -> U256 {
        let test_call_contract_package = test_context
            .test_call_package_hash_key
            .into_hash()
            .map(ContractPackageHash::new)
            .expect("should be package hash");

        let lender_contract_package = test_context
            .lender_package_hash_key
            .into_hash()
            .map(ContractPackageHash::new)
            .expect("should be hash");
        let deploy = DeployItemBuilder::new()
            .with_address(test_context.account_address)
            .with_stored_versioned_contract_by_name(
                "test_call_package_hash",
                None,
                "get_flash_fee",
                runtime_args! {
                    "token" => test_context.erc20_package_hash_key,
                    "lender" => lender_contract_package,
                    "amount" => amount
                },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[test_context.account_address])
            .with_deploy_hash([42; 32])
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy).build();
        builder.exec(execute_request).commit().expect_success();

        get_test_result(builder, test_call_contract_package)
    }

    #[test]
    fn test_flash_borrow_success() {
        let (mut builder, test_context) = setup();

        // =============== transfer ERC20 token to lender start ===============
        let to_lender = true;
        let amount_lender_before = U256::from(50000u128);

        tranfer_erc20(&mut builder, &test_context, to_lender, amount_lender_before);

        let balance_lender_before = balance_of(&mut builder, &test_context, to_lender);
        // get original balance of lender *before* flash borrow
        assert_eq!(amount_lender_before, balance_lender_before);
        // =============== transfer ERC20 token to lender end ===============

        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee start ===============
        let to_lender = false;
        let amount_borrower_before = U256::from(50000u128);
        tranfer_erc20(
            &mut builder,
            &test_context,
            to_lender,
            amount_borrower_before,
        );
        //  get balance of borrower *before* flash borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context, to_lender);
        assert_eq!(amount_borrower_before, balance_borrower_before);
        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee end ===============

        // ===============  flash_borrow start ==========================
        let amount = U256::from(4000u128);
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit().expect_success();
        // ===============  flash_borrow end ==========================

        // ===============  check balance after flash borrow start======
        // get flash fee
        let flash_fee = flash_fee(&mut builder, &test_context, amount);

        // get balance of lender *after* flash borrow
        let to_lender = true;
        let balance_lender_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of lender should be added flash fee
        assert_eq!(amount_lender_before + flash_fee, balance_lender_after);

        // get balance of borrower *after* flash borrow
        let to_lender = false;
        let balance_borrower_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of borrower should be minus flash fee
        assert_eq!(amount_borrower_before - flash_fee, balance_borrower_after);
        // ===============  check balance after flash borrow end======
    }

    #[test]
    fn test_flash_borrow_insufficent_balance_lender() {
        let (mut builder, test_context) = setup();

        // =============== transfer ERC20 token to lender start ===============
        let to_lender = true;
        let amount_lender_before = U256::from(5000u128);

        tranfer_erc20(&mut builder, &test_context, to_lender, amount_lender_before);

        let balance_lender_before = balance_of(&mut builder, &test_context, to_lender);
        // get original balance of lender *before* flash borrow
        assert_eq!(amount_lender_before, balance_lender_before);
        // =============== transfer ERC20 token to lender end ===============

        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee start ===============
        let to_lender = false;
        let amount_borrower_before = U256::from(5000u128);
        tranfer_erc20(
            &mut builder,
            &test_context,
            to_lender,
            amount_borrower_before,
        );
        //  get balance of borrower *before* flash borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context, to_lender);
        assert_eq!(amount_borrower_before, balance_borrower_before);
        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee end ===============

        // ===============  flash_borrow start ==========================
        let amount = U256::from(5001u128);
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit();

        let error = builder.get_error().expect("should have error");
        assert!(
            matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_INSUFFICIENT_BALANCE),
            "{:?}",
            error
        );
        // ===============  flash_borrow end ==========================

        // ===============  check balance after flash borrow start======

        // get balance of lender *after* flash borrow
        let to_lender = true;
        let balance_lender_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of lender shouldn't be changed
        assert_eq!(amount_lender_before, balance_lender_after);

        // get balance of borrower *after* flash borrow
        let to_lender = false;
        let balance_borrower_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of borrower shouldn't be changed
        assert_eq!(amount_borrower_before, balance_borrower_after);
        // ===============  check balance after flash borrow end======
    }

    #[test]
    fn test_flash_borrow_insufficent_balance_borrower() {
        // borrower balance cannot cover flash fee
        let (mut builder, test_context) = setup();

        // =============== transfer ERC20 token to lender start ===============
        let to_lender = true;
        let amount_lender_before = U256::from(5000u128);

        tranfer_erc20(&mut builder, &test_context, to_lender, amount_lender_before);

        let balance_lender_before = balance_of(&mut builder, &test_context, to_lender);
        // get original balance of lender *before* flash borrow
        assert_eq!(amount_lender_before, balance_lender_before);
        // =============== transfer ERC20 token to lender end ===============

        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee start ===============
        let to_lender = false;
        let amount_borrower_before = U256::from(1u128);
        tranfer_erc20(
            &mut builder,
            &test_context,
            to_lender,
            amount_borrower_before,
        );
        //  get balance of borrower *before* flash borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context, to_lender);
        assert_eq!(amount_borrower_before, balance_borrower_before);
        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee end ===============

        // ===============  flash_borrow start ==========================
        let amount = U256::from(4000u128);
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit();

        let error = builder.get_error().expect("should have error");
        assert!(
            matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_INSUFFICIENT_BALANCE),
            "{:?}",
            error
        );
        // ===============  flash_borrow end ==========================

        // ===============  check balance after flash borrow start======

        // get balance of lender *after* flash borrow
        let to_lender = true;
        let balance_lender_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of lender shouldn't be changed
        assert_eq!(balance_lender_before, balance_lender_after);

        // get balance of borrower *after* flash borrow
        let to_lender = false;
        let balance_borrower_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of borrower shouldn't be changed
        assert_eq!(balance_borrower_before, balance_borrower_after);
        // ===============  check balance after flash borrow end======
    }

    #[test]
    fn test_flash_borrow_borrowamount_overflow() {
        // test max borrow amount
        let (mut builder, test_context) = setup();

        // =============== transfer ERC20 token to lender start ===============
        let to_lender = true;
        let amount_lender_before = U256::from(5000u128);

        tranfer_erc20(&mut builder, &test_context, to_lender, amount_lender_before);

        let balance_lender_before = balance_of(&mut builder, &test_context, to_lender);
        // get original balance of lender *before* flash borrow
        assert_eq!(amount_lender_before, balance_lender_before);
        // =============== transfer ERC20 token to lender end ===============

        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee start ===============
        let to_lender = false;
        let amount_borrower_before = U256::from(5000u128);
        tranfer_erc20(
            &mut builder,
            &test_context,
            to_lender,
            amount_borrower_before,
        );
        //  get balance of borrower *before* flash borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context, to_lender);
        assert_eq!(amount_borrower_before, balance_borrower_before);
        // =============== transfer erc20 token to borrower ===============
        // =============== for covering flash fee end ===============

        // ===============  flash_borrow start ==========================
        let amount = U256::max_value();
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit();

        let error = builder.get_error().expect("should have error");
        assert!(
            matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_FFLASH_LENDER_OVERFLOW),
            "{:?}",
            error
        );
        // ===============  flash_borrow end ==========================

        // ===============  check balance after flash borrow start======

        // get balance of lender *after* flash borrow
        let to_lender = true;
        let balance_lender_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of lender shouldn't be changed
        assert_eq!(amount_lender_before, balance_lender_after);

        // get balance of borrower *after* flash borrow
        let to_lender = false;
        let balance_borrower_after = balance_of(&mut builder, &test_context, to_lender);

        //balance of borrower shouldn't be changed
        assert_eq!(amount_borrower_before, balance_borrower_after);
        // ===============  check balance after flash borrow end======
    }

    #[test]
    fn test_get_max_flash_loan() {
        let (mut builder, test_context) = setup();
        let to_lender = true;
        let amount = U256::from(9990000u128);
        tranfer_erc20(&mut builder, &test_context, to_lender, amount);

        let max_flash_loan = max_flash_loan(&mut builder, &test_context);

        // get balance of lender
        let to_lender = true;
        let balance_lender = balance_of(&mut builder, &test_context, to_lender);
        // balance of lender should be equal to max flash loan
        assert_eq!(balance_lender, max_flash_loan);
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
