/**
 * @fileOverview CSPR JS SDK demo: CASK - install contract.
 */

import {
  CasperClient,
  DeployUtil,
  RuntimeArgs,
  CLString,
  CLMap,
  CLByteArray,
  CLAccountHash,
  CLPublicKey,
  CLKey,
  CLU256
} from "casper-js-sdk";
import * as constants from "../constants";
import * as utils from "../utils";

// Path to contract to be installed.
const PATH_TO_CONTRACT = constants.PATH_TO_CONTRACT_BORROW;


//**************************** for lender_package start*******************************/
const hexString2 =
  "5b54b5479519a379d8de2c0b92ffc9ead967e5d539a79c2b1c7ce84e5784434";

const hex2 = Uint8Array.from(Buffer.from(hexString2, "hex"));

const LENDER_ADDRESS = new CLKey(new CLByteArray(hex2));
//**************************** for lender_package end*******************************/

// const hash = "5b54b5479519a379d8de2c0b92ffc9ead967e5d539a79c2b1c7ce84e5784434"
// const LENDER_ADDRESS = new CLByteArray(Uint8Array.from(Buffer.from(hash, 'hex')));



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
        lender_address: LENDER_ADDRESS,
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
installed contract -> Borrower
... account = ${constants.PATH_TO_SOURCE_KEYS}
... deploy chain = ${constants.DEPLOY_CHAIN_NAME}
... deploy dispatch node = ${constants.DEPLOY_NODE_ADDRESS}
... deploy gas payment = ${constants.DEPLOY_GAS_PAYMENT_FOR_INSTALL}
... deploy gas price = ${constants.DEPLOY_GAS_PRICE}
contract installation details:
... path = ${PATH_TO_CONTRACT}
... deploy hash = ${deployHash}
---------------------------------------------------------------------
    `);
};

main();
