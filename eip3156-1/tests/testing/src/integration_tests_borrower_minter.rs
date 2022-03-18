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
        runtime_args, ApiError, CLTyped, ContractHash, ContractPackageHash, Key, Motes, PublicKey,
        RuntimeArgs, SecretKey, U256, U512,
    };

    // const ERC20_WASM: &str =
    //     "/home/jh/caspereco/erc20/target/wasm32-unknown-unknown/release/erc20_token.wasm";
    const MINTER_WASM: &str =
        "/home/jh/mywork/eip3156workspace/eip3156-1/target/wasm32-unknown-unknown/release/minter.wasm";
    const BORROWER_WASM: &str = "borrower.wasm";
    const TEST_CALL_WASM: &str = "eip3156_test_call.wasm";

    const MINTER_ADDRESS: &str = "lender_address";

    const BORROWER_PACKAGE_HASH_KEY: &str = "BORROWER";

    const MINTER_CONTRACT_KEY: &str = "erc20_token_contract";
    const MINTER_PACKAGE_KEY: &str = "erc20_package_hash";
    const TOTAL_SUPPLY_KEY: &str = "total_supply";

    const ERROR_INSUFFICIENT_BALANCE: u16 = u16::MAX - 111;
    const ERROR_OVERFLOW: u16 = u16::MAX - 112;

    struct TestFixture {
        account_address: AccountHash,
        test_call_package_hash_key: Key,
        minter_contract_hash: ContractHash,
        minter_package_hash_key: Key,
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

    fn setup(total_supply: U256) -> (InMemoryWasmTestBuilder, TestFixture) {
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

        // ====install minter(improved erc20) contract start=========//
        let exec_request = {
            ExecuteRequestBuilder::standard(
                account_address,
                MINTER_WASM,
                runtime_args! {
                "name"=> String::from("ORANGE"),
                "symbol" => String::from("OOO"),
                "decimals" => 10u8,
                "fee" => U256::from(10u8),
                "total_supply" => total_supply
                },
            )
            .build()
        };

        test_builder.exec(exec_request).expect_success().commit();

        // ======install minter(improved erc20) contract end =========//

        //get account
        let account = test_builder
            .query(None, Key::Account(account_address), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        //get minter(improved erc20) package hash
        let minter_package_hash = account
            .named_keys()
            .get(MINTER_PACKAGE_KEY)
            .expect("should have minter(improved erc20) package");

        // ======install borrower contract start=========//
        let exec_request = {
            ExecuteRequestBuilder::standard(
                account_address,
                BORROWER_WASM,
                runtime_args! {
                    MINTER_ADDRESS => *minter_package_hash
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

        let minter_package_hash_key = *minter_package_hash;

        let borrower_package_hash_key = *(account
            .named_keys()
            .get(BORROWER_PACKAGE_HASH_KEY)
            .expect("should have borrower package hash"));

        // =======install test-call start ============
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

        // get minter contract hash key
        let minter_contract_hash = account
            .named_keys()
            .get(MINTER_CONTRACT_KEY)
            .and_then(|key| key.into_hash())
            .map(ContractHash::new)
            .expect("should have minter contract hash");

        let test_context = TestFixture {
            account_address,
            test_call_package_hash_key,
            minter_contract_hash,
            minter_package_hash_key,
            borrower_package_hash_key,
        };
        (test_builder, test_context)
    }

    fn tranfer_erc20(
        builder: &mut InMemoryWasmTestBuilder,
        test_context: &TestFixture,
        amount: U256,
    ) {
        let deploy = DeployItemBuilder::new()
            .with_address(test_context.account_address)
            .with_stored_session_named_key(
                MINTER_CONTRACT_KEY,
                "transfer",
                runtime_args! {
                    "recipient" =>  test_context.borrower_package_hash_key,
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
                    "token" => test_context.minter_package_hash_key,
                    "amount" => amount,
                },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[test_context.account_address])
            .with_deploy_hash([42; 32])
            .build();
        ExecuteRequestBuilder::from_deploy_item(deploy).build()
    }

    fn balance_of(builder: &mut InMemoryWasmTestBuilder, test_context: &TestFixture) -> U256 {
        //get balance_uref
        let balance_uref = *builder
            .query(
                None,
                Key::Account(test_context.account_address),
                &[MINTER_CONTRACT_KEY.to_string()],
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

        let dic_item_key =
            base64::encode(test_context.borrower_package_hash_key.to_bytes().unwrap());

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

        let minter_contract_package = test_context
            .minter_package_hash_key
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
                    "token" => test_context.minter_package_hash_key,
                    "lender" => minter_contract_package,
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
        let inital_total_supply = U256::from(2220000000u128);
        let (mut builder, test_context) = setup(inital_total_supply);
        // transfer erc20 token to borrower
        let amount = U256::from(222u128);

        tranfer_erc20(&mut builder, &test_context, amount);
        // get balance of borrower before borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context);
        assert_eq!(amount, balance_borrower_before);

        // get total supply before flash borrow
        let total_supply_before: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);

        // flash_borrow
        let amount = U256::from(22200u128);
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit().expect_success();

        // flash_borrow(&mut builder, &test_context, amount);
        // get flash fee
        let flash_fee = flash_fee(&mut builder, &test_context, amount);

        // get balance of borrower after borrow
        let balance_borrower_after = balance_of(&mut builder, &test_context);
        assert_eq!(balance_borrower_before - flash_fee, balance_borrower_after);

        // get totalsupply of minter after borrow
        let total_supply_after: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);
        assert_eq!(total_supply_before - flash_fee, total_supply_after);
    }
    #[test]
    fn test_flash_borrow_borrowamount_overflow() {
        // test max borrow amount
        let inital_total_supply = U256::from(2220000000u128);
        let (mut builder, test_context) = setup(inital_total_supply);
        // transfer erc20 token to borrower
        let amount = U256::from(222u128);

        tranfer_erc20(&mut builder, &test_context, amount);
        // get balance of borrower before borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context);
        assert_eq!(amount, balance_borrower_before);

        // get total supply before flash borrow
        let total_supply_before: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);

        // flash_borrow
        let amount = U256::MAX;
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit();

        let error = builder.get_error().expect("should have error");
        assert!(
            matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_OVERFLOW),
            "{:?}",
            error
        );

        // get balance of borrower after borrow
        let balance_borrower_after = balance_of(&mut builder, &test_context);
        assert_eq!(balance_borrower_before, balance_borrower_after);

        // get totalsupply of minter after borrow
        let total_supply_after: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);
        assert_eq!(total_supply_before, total_supply_after);
    }
    #[test]
    fn test_flash_borrow_initialsupply_overflow() {
        // test max intial supply amount
        let inital_total_supply = U256::max_value();
        let (mut builder, test_context) = setup(inital_total_supply);
        // transfer erc20 token to borrower
        let amount = U256::from(22u128);

        tranfer_erc20(&mut builder, &test_context, amount);
        // get balance of borrower before borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context);
        assert_eq!(amount, balance_borrower_before);

        // get total supply before flash borrow
        let total_supply_before: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);

        // flash_borrow
        let amount = U256::from(1u128);
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit();
        let error = builder.get_error().expect("should have error");
        assert!(
            matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_OVERFLOW),
            "{:?}",
            error
        );

        // get balance of borrower after borrow
        let balance_borrower_after = balance_of(&mut builder, &test_context);
        //balance of borrower shouldn't be changed
        assert_eq!(balance_borrower_before, balance_borrower_after);

        // get totalsupply of minter after borrow
        let total_supply_after: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);
        //balance of total_supply shouldn't be changed
        assert_eq!(total_supply_before, total_supply_after);
    }

    #[test]
    fn test_flash_borrow_insufficent_balance_borrower() {
        let inital_total_supply = U256::from(2220000000u128);
        let (mut builder, test_context) = setup(inital_total_supply);
        // transfer erc20 token to borrower
        let amount = U256::from(22u128);

        tranfer_erc20(&mut builder, &test_context, amount);
        // get balance of borrower before borrow
        let balance_borrower_before = balance_of(&mut builder, &test_context);
        assert_eq!(amount, balance_borrower_before);

        // get total supply before flash borrow
        let total_supply_before: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);

        // flash_borrow
        let amount = U256::from(23200u128);
        let execute_request = make_flash_borrow_request(&test_context, amount);

        builder.exec(execute_request).commit();
        let error = builder.get_error().expect("should have error");
        assert!(
            matches!(error, CoreError::Exec(ExecError::Revert(ApiError::User(user_error))) if user_error == ERROR_INSUFFICIENT_BALANCE),
            "{:?}",
            error
        );

        // get balance of borrower after borrow
        let balance_borrower_after = balance_of(&mut builder, &test_context);
        //balance of borrower shouldn't be changed
        assert_eq!(balance_borrower_before, balance_borrower_after);

        // get totalsupply of minter after borrow
        let total_supply_after: U256 =
            builder.get_value(test_context.minter_contract_hash, TOTAL_SUPPLY_KEY);
        //balance of total_supply shouldn't be changed
        assert_eq!(total_supply_before, total_supply_after);
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
