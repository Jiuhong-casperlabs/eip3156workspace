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
    use casper_types::{
        account::AccountHash, bytesrepr::ToBytes, runtime_args, CLValue, ContractPackageHash, Key,
        Motes, PublicKey, RuntimeArgs, SecretKey, URef, U256, U512,
    };

    const ERC20_WASM: &str =
        "/home/jh/caspereco/erc20/target/wasm32-unknown-unknown/release/erc20_token.wasm";
    const LENDER_WASM: &str = "lender.wasm";
    const BORROWER_WASM: &str = "borrower.wasm";
    const ARG_INITIAL_SUPPORTED_TOKENS: &str = "initial_supported_tokens";

    const LENDER_ADDRESS: &str = "lender_address";

    const LENDER_PACKAGE_HASH_KEY: &str = "LENDER";

    const BORROWER_PACKAGE_HASH_KEY: &str = "BORROWER";

    const ERC20_PACKAGE_KEY: &str = "erc20_token_contract_1";

    struct TestFixture {
        test_builder: InMemoryWasmTestBuilder,
        account_address: AccountHash,
        erc20_package_hash_key: Key,
        lender_package_hash_key: Key,
        borrower_package_hash_key: Key,
    }

    impl TestFixture {
        /// Initialize the test fixture by setting up a genesis account, running the genesis request
        /// and installing the counter smart contract.
        fn deploy() -> Self {
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

            // ========= install erc20 contract start========= //
            let erc20_installer_session_code = ERC20_WASM;
            let erc20_installer_session_args = runtime_args! {
            "name"=> String::from("ORANGE"),
            "symbol" => String::from("OOO"),
            "decimals" => 10u8,
            "total_supply" => U256::from(1000000000000000u128)
            };
            let installer_payment_args = runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            };
            let deploy_item1 = DeployItemBuilder::new()
                .with_empty_payment_bytes(installer_payment_args)
                .with_session_code(erc20_installer_session_code, erc20_installer_session_args)
                .with_authorization_keys(&[account_address])
                .with_address(account_address)
                .build();

            let execute_request1 = ExecuteRequestBuilder::from_deploy_item(deploy_item1).build();

            test_builder
                .exec(execute_request1)
                .commit()
                .expect_success();

            // ========= install erc20 contract end========= //

            // ========= install lender contract start========= //
            //get account
            let account = test_builder
                .query(None, Key::Account(account_address), &[])
                .expect("should query account")
                .as_account()
                .cloned()
                .expect("should be account");

            //get erc20 package hash
            let erc20_package_hash = account
                .named_keys()
                .get(ERC20_PACKAGE_KEY)
                .expect("should have erc20 package");

            // install lender
            let lender_installer_session_code = LENDER_WASM;
            let lender_installer_session_args = runtime_args! {
                ARG_INITIAL_SUPPORTED_TOKENS => vec![(*erc20_package_hash, U256::from(10))]
            };
            let installer_payment_args = runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            };
            let deploy_item2 = DeployItemBuilder::new()
                .with_empty_payment_bytes(installer_payment_args)
                .with_session_code(lender_installer_session_code, lender_installer_session_args)
                .with_authorization_keys(&[account_address])
                .with_address(account_address)
                .build();

            let execute_request2 = ExecuteRequestBuilder::from_deploy_item(deploy_item2).build();

            test_builder
                .exec(execute_request2)
                .commit()
                .expect_success();

            // ========= install lender contract end========= //

            // ========= install borrower contract start========= //
            //get account
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

            // install borrower
            let borrower_installer_session_code = BORROWER_WASM;
            let borrower_installer_session_args = runtime_args! {
                LENDER_ADDRESS => *lender_package_hash
            };
            let installer_payment_args = runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            };
            let deploy_item3 = DeployItemBuilder::new()
                .with_empty_payment_bytes(installer_payment_args)
                .with_session_code(
                    borrower_installer_session_code,
                    borrower_installer_session_args,
                )
                .with_authorization_keys(&[account_address])
                .with_address(account_address)
                .build();

            let execute_request3 = ExecuteRequestBuilder::from_deploy_item(deploy_item3).build();

            test_builder
                .exec(execute_request3)
                .commit()
                .expect_success();

            // ========= install borrower contract end========= //

            // get account
            let account = test_builder
                .query(None, Key::Account(account_address), &[])
                .expect("should query account")
                .as_account()
                .cloned()
                .expect("should be account");

            // get named keys

            let named_keys = account.named_keys();
            // println!("named keys are {:?}", named_keys);

            let erc20_package_hash_key = *(account
                .named_keys()
                .get(ERC20_PACKAGE_KEY)
                .expect("should have package hash"));

            let lender_package_hash_key = *(account
                .named_keys()
                .get(LENDER_PACKAGE_HASH_KEY)
                .expect("should have package hash"));

            let borrower_package_hash_key = *(account
                .named_keys()
                .get(BORROWER_PACKAGE_HASH_KEY)
                .expect("should have package hash"));
            let test_context = Self {
                test_builder,
                account_address,
                erc20_package_hash_key,
                lender_package_hash_key,
                borrower_package_hash_key,
            };

            test_context
        }

        fn tranfer_erc20(&mut self, to_lender: bool, amount: U256) {
            // get erc20 package hash
            let erc20_package_hash = self
                .erc20_package_hash_key
                .into_hash()
                .map(ContractPackageHash::new)
                .expect("should be hash");

            let deploy = DeployItemBuilder::new()
                .with_address(self.account_address)
                .with_stored_versioned_contract_by_hash(
                    erc20_package_hash.value(),
                    None,
                    "transfer",
                    runtime_args! {
                        "recipient" =>if to_lender {self.lender_package_hash_key} else {self.borrower_package_hash_key},
                        "amount" => amount
                    },
                )
                .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
                .with_authorization_keys(&[self.account_address])
                .with_deploy_hash([42; 32])
                .build();
            ExecuteRequestBuilder::new().push_deploy(deploy).build();

            // get balance
            let balance_uref = self
                .test_builder
                .query(
                    None,
                    Key::Account(self.account_address),
                    &[ERC20_PACKAGE_KEY.to_string()],
                )
                .expect("should have validator slots")
                .as_contract()
                .expect("should be contract")
                .clone()
                .take_named_keys()
                .get("balances")
                .unwrap()
                .clone()
                .as_uref()
                .unwrap()
                .clone();
            println!("balance_uref is {:?}", balance_uref);

            let lender = if to_lender {
                self.lender_package_hash_key
            } else {
                self.borrower_package_hash_key
            };

            let item_key = self
                .test_builder
                .query(
                    None,
                    Key::Account(self.account_address),
                    &[ERC20_PACKAGE_KEY.to_string()],
                )
                .expect("should have validator slots")
                .as_contract()
                .unwrap()
                .clone()
                .take_named_keys();
            //names keys under erc20 contract
            println!("=================");
            println!("item_keys is {:?}", item_key);
            println!("=================");
            let account = self
                .test_builder
                .query(None, Key::Account(self.account_address), &[])
                .expect("should query account")
                .as_account()
                .cloned()
                .expect("should be account");

            // get named keys

            let named_keys = account.named_keys();
            println!("named keys under account is {:?}", named_keys);

            // let item_key = base64::encode(&lender.to_bytes().unwrap());
            // let item_key: String = self
            //     .test_builder
            //     .query(
            //         None,
            //         Key::Account(self.account_address),
            //         &[
            //             ERC20_PACKAGE_KEY.to_string(),
            //             "balance_item_key".to_string(),
            //         ],
            //     )
            //     .expect("should have validator slots")
            //     .as_cl_value()
            //     .unwrap()
            //     .clone()
            //     .into_t()
            //     .unwrap();

            // let value: U256 = self
            //     .test_builder
            //     .query_dictionary_item(None, balance_uref, &item_key)
            //     .ok()
            //     .unwrap()
            //     .as_cl_value()
            //     .unwrap()
            //     .clone()
            //     .into_t()
            //     .unwrap();
            // print!("{}", value);
        }

        fn flash_borrow(&mut self) {
            // get borrower package hash
            let borrower_package_hash = self
                .borrower_package_hash_key
                .into_hash()
                .map(ContractPackageHash::new)
                .expect("should be hash");
            let deploy = DeployItemBuilder::new()
                .with_address(self.account_address)
                .with_stored_versioned_contract_by_hash(
                    borrower_package_hash.value(),
                    None,
                    "flash_borrow",
                    runtime_args! {
                        "token" => self.erc20_package_hash_key,
                        "amount" => U256::from(2222),
                    },
                )
                .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
                .with_authorization_keys(&[self.account_address])
                .with_deploy_hash([42; 32])
                .build();
            ExecuteRequestBuilder::new().push_deploy(deploy).build();
        }

        pub fn balance_of(&self) {
            //get balance_uref
            let balance_uref = self
                .test_builder
                .query(
                    None,
                    Key::Account(self.account_address),
                    &[ERC20_PACKAGE_KEY.to_string()],
                )
                .expect("should have validator slots")
                .as_contract()
                .expect("should be contract")
                .clone()
                .take_named_keys()
                .get("balances")
                .unwrap()
                .clone()
                .as_uref()
                .unwrap()
                .clone();
            println!("balance_uref is {:?}", balance_uref);
            // .as_cl_value()
            // .expect("should be CLValue")
            // .clone()
            // .into_t()
            // .expect("should be u32");
            // let a: URef = CLValue::into_t();

            // println!("value: {:?}", balance_uref);
            // .as_uref()
            // .cloned()
            // .expect("should be account");

            // let item_key = base64::encode(&account.to_bytes().unwrap());
            let lender = self.borrower_package_hash_key;

            let item_key = base64::encode(&lender.to_bytes().unwrap());
            println!("item_key is: {}", item_key);
            println!("lender is {}", lender);

            let value: U256 = self
                .test_builder
                .query_dictionary_item(None, balance_uref, &item_key)
                .ok()
                .unwrap()
                .as_cl_value()
                .unwrap()
                .clone()
                .into_t()
                .unwrap();
            print!("{}", value);

            // Some(value.into_t::<U256>().unwrap())
        }

        fn get_balance(&mut self, from_lender: bool) {
            // get erc20 package hash
            let erc20_package_hash = self
                .erc20_package_hash_key
                .into_hash()
                .map(ContractPackageHash::new)
                .expect("should be hash");

            let deploy = DeployItemBuilder::new()
                .with_address(self.account_address)
                .with_stored_versioned_contract_by_hash(
                    erc20_package_hash.value(),
                    None,
                    "balance_of",
                    runtime_args! {
                        "address" =>if from_lender {self.lender_package_hash_key} else {self.borrower_package_hash_key},
                    },
                )
                .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
                .with_authorization_keys(&[self.account_address])
                .with_deploy_hash([42; 32])
                .build();
            let a = ExecuteRequestBuilder::new().push_deploy(deploy).build();
        }
    }

    #[test]
    fn transfer_token_to_lender() {
        // transfer 50000 to lender
        let to_lender = true;
        let amount = U256::from(50000);
        let mut a = TestFixture::deploy();
        let b = a.tranfer_erc20(to_lender, amount);

        // get balance of lender
        a.balance_of();

        //transfer 20 to borrower for flashfee
        // let to_lender = false;
        // let amount = U256::from(10);
        // a.tranfer_erc20(to_lender, amount);

        //invoke flash_borrow of borrower
        // TestFixture::deploy().flash_borrow();

        // get lender balance

        // get borrower balance
    }

    #[test]
    fn test_balance() {
        // TestFixture::deploy().balance_of();
    }
    // #[test]
    // fn should_increment_with_direct_call() {
    //     let mut fixture = TestFixture::deploy();
    //     let use_stored_session = true;
    //     for expected_value in 1..=3 {
    //         fixture.increment_counter(use_stored_session);
    //         assert_eq!(fixture.get_counter(), expected_value);
    //     }
    // }

    // #[test]
    // fn should_increment_with_counter_call_contract() {
    //     let mut fixture = TestFixture::deploy();
    //     let use_stored_session = false;
    //     for expected_value in 1..=3 {
    //         fixture.increment_counter(use_stored_session);
    //         assert_eq!(fixture.get_counter(), expected_value);
    //     }
    // }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
