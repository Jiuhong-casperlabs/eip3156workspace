## borrower <--> lender

 ========== ========== step1: install erc20  ========== ========== 
```
casper-client put-deploy --node-address http://16.162.124.124:7777  \
--chain-name casper-test \
--secret-key /home/jh/keys/test77/secret_key.pem \
--payment-amount 70000000000 \
--session-path target/wasm32-unknown-unknown/release/erc20_token.wasm  \
--session-arg "name:String='ORANGE'" \
--session-arg "symbol:String='OOO'" \
--session-arg "decimals:u8='10'" \
--session-arg "total_supply:u256='1000000000000000'" 
```
write down *\<erc20 contract package hash\>*


 ========== ========== step2 install lender contract (LENDER) =============

[js client](./eip3156clientjs/README.md) -> install lender contract (lender)

write down <*lender contract package hash*>

 ========== ========== step3 transfer tokens to LENDER  =========

```
casper-client put-deploy --node-address http://16.162.124.124:7777  \
--chain-name casper-test \
--secret-key /home/jh/keys/test77/secret_key.pem \
--payment-amount 10000000000 \
--session-package-hash <erc20 contract package hash>  \
--session-entry-point "transfer" \
--session-arg "recipient:key='<lender contract package hash>'" \
--session-arg "amount:u256='50000'" 
```

 ========== ========== step4 install borrower contract ==========
```
casper-client put-deploy --node-address http://16.162.124.124:7777  \
--chain-name casper-test \
--secret-key /home/jh/keys/test77/secret_key.pem \
--payment-amount 70000000000 \
--session-path /home/jh/mywork/eip3156workspace/eip3156-1/target/wasm32-unknown-unknown/release/borrower.wasm  \
--session-arg "lender_address:Key='<lender contract package hash>'"
``` 

write down *borrower contract package hash*

 ========== step5 transfer tokens to borrower for covering flashfee=========

```
casper-client put-deploy --node-address http://16.162.124.124:7777  \
--chain-name casper-test \
--secret-key /home/jh/keys/test77/secret_key.pem \
--payment-amount 10000000000 \
--session-package-hash <erc20 contract package hash>  \
--session-entry-point "transfer" \
--session-arg "recipient:key='<borrower contract package hash>'" \
--session-arg "amount:u256='3'" 
```


 ========== ==========  step6 invoke flash_borrow of borrower ========
```
casper-client put-deploy --chain-name casper-test \
-n http://16.162.124.124:7777 \
--secret-key /home/jh/keys/test77/secret_key.pem \
--payment-amount 5000000000 \
--session-package-hash <borrower contract package hash>  \
--session-entry-point "flash_borrow" \
--session-arg "token:key='<erc20 contract package hash>'" \
--session-arg "amount:u256='2222'"
```
---
## borrower <--> minter

========== ========== step1: install minter(improved erc20)  ========== ========== 

```
casper-client put-deploy --node-address http://16.162.124.124:7777  \
--chain-name casper-test \
--secret-key /home/jh/keys/jdk1/secret_key.pem \
--payment-amount 110000000000 \
--session-path target/wasm32-unknown-unknown/release/minter.wasm  \
--session-arg "name:String='ORANGE'" \
--session-arg "symbol:String='OOO'" \
--session-arg "fee:U256='10'" \
--session-arg "decimals:u8='10'" \
--session-arg "total_supply:u256='1000000000000000'" 
```

write down <*minter contract package hash*>


 ========== ========== step2: install borrower =========

```
casper-client put-deploy --node-address http://16.162.124.124:7777  \
--chain-name casper-test \
--secret-key /home/jh/keys/jdk1/secret_key.pem \
--payment-amount 70000000000 \
--session-path /home/jh/mywork/eip3156workspace/eip3156-1/target/wasm32-unknown-unknown/release/borrower.wasm  \
--session-arg "lender_address:Key='<minter contract package hash>'" 
```

write down <*borrower contract package hash*>

 ========== ========== step3 transfer tokens to borrower for flashfee =========

```
casper-client put-deploy --node-address http://16.162.124.124:7777  \
--chain-name casper-test \
--secret-key /home/jh/keys/jdk1/secret_key.pem \
--payment-amount 10000000000 \
--session-package-hash <minter contract package hash> \
--session-entry-point "transfer" \
--session-arg "recipient:key='<borrower contract package hash>'" \
--session-arg "amount:u256='3'" 
```

 ========== ==========  step 4 invoke flash_borrow of borrower========

```
casper-client put-deploy --chain-name casper-test \
-n http://16.162.124.124:7777 \
--secret-key /home/jh/keys/jdk1/secret_key.pem \
--payment-amount 5000000000 \
--session-package-hash <borrower contract package hash>  \
--session-entry-point "flash_borrow" \
--session-arg "token:key='<minter contract package hash>'" \
--session-arg "amount:u256='2222'"
```
