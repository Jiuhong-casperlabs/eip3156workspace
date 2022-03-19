/**
 * @fileOverview CSPR JS SDK demo: CASK - install contract.
 */

import {
  CasperClient,
  DeployUtil,
  RuntimeArgs,
  CLByteArray,
  CLKey,
  CLU256,
  CLTuple2,
  CLList,
} from "casper-js-sdk";
import * as constants from "../constants";
import * as utils from "../utils";

// Path to contract to be installed.
const PATH_TO_CONTRACT = constants.PATH_TO_CONTRACT_LEND;

//**************************** for token1_package_has start*******************************/
const tokenpackageString1 =
  "17202d448a32af52252a21c8296c9562c10a1f3da69efc5a5d01678aac753b7e";

const hex1 = Uint8Array.from(Buffer.from(tokenpackageString1, "hex"));

const token1_address = new CLKey(new CLByteArray(hex1));
//**************************** for token1_package_has end*******************************/

//**************************** for token2_package_has start*******************************/
const tokenpackageString2 =
  "f27e4d26f43d64a9e0688f0d90f4c129e50f930b0d46416af1f1c9a18f957dbb";

const hex2 = Uint8Array.from(Buffer.from(tokenpackageString2, "hex"));

const token2_address = new CLKey(new CLByteArray(hex2));
//**************************** for token2_package_has end*******************************/

const token1_fee = new CLU256(10);
const token1 = new CLTuple2([token1_address, token1_fee]);

const token2_fee = new CLU256(20);
const token2 = new CLTuple2([token2_address, token2_fee]);

const SUPPORTED_TOKENS = new CLList([token1, token2]);
// Vec<(Address, U256)>
/**
 * Demonstration entry point.
 */
const main = async () => {
  // Step 1: Set casper node client.
  const client = new CasperClient(constants.DEPLOY_NODE_ADDRESS);

  // Step 2: Set contract operator key pair.
  const keyPairOfContract = utils.getKeyPairOfContract(
    constants.PATH_TO_SOURCE_KEYS
  );

  // Step 3: Set contract installation deploy (unsigned).
  let deploy = DeployUtil.makeDeploy(
    new DeployUtil.DeployParams(
      keyPairOfContract.publicKey,
      constants.DEPLOY_CHAIN_NAME,
      constants.DEPLOY_GAS_PRICE,
      constants.DEPLOY_TTL_MS
    ),
    DeployUtil.ExecutableDeployItem.newModuleBytes(
      utils.getBinary(PATH_TO_CONTRACT),
      RuntimeArgs.fromMap({
        initial_supported_tokens: SUPPORTED_TOKENS,
      })
    ),
    DeployUtil.standardPayment(constants.DEPLOY_GAS_PAYMENT_FOR_INSTALL)
  );

  // Step 4: Sign deploy.
  deploy = client.signDeploy(deploy, keyPairOfContract);

  // Step 5: Dispatch deploy to node.
  const deployHash = await client.putDeploy(deploy);

  // Step 6: Render deploy details.
  logDetails(deployHash);
};

/**
 * Emits to stdout deploy details.
 * @param {String} deployHash - Identifer of dispatched deploy.
 */
const logDetails = (deployHash) => {
  console.log(`
---------------------------------------------------------------------
installed contract -> LEND
... account = ${constants.PATH_TO_SOURCE_KEYS}
... deploy chain = ${constants.DEPLOY_CHAIN_NAME}
... deploy dispatch node = ${constants.DEPLOY_NODE_ADDRESS}
... deploy gas payment = ${constants.DEPLOY_GAS_PAYMENT_FOR_INSTALL}
contract installation details:
... path = ${PATH_TO_CONTRACT}
... deploy hash = ${deployHash}
---------------------------------------------------------------------
    `);
};

main();
