// Path to lend contract
export const PATH_TO_CONTRACT_LEND = `${process.env.HOME}/mywork/eip3156workspace/eip3156-1/target/wasm32-unknown-unknown/release/lender.wasm`;

// Path to source keys
export const PATH_TO_SOURCE_KEYS = `${process.env.HOME}/keys/jdk1`;

// Name of target chain.
export const DEPLOY_CHAIN_NAME =
  // process.env.CSPR_INTS_DEPLOY_CHAIN_NAME || "casper-net-1";
  process.env.CSPR_INTS_DEPLOY_CHAIN_NAME || "casper-test";

// Gas payment to be offered.
export const DEPLOY_GAS_PAYMENT_FOR_INSTALL =
  process.env.CSPR_INTS_DEPLOY_GAS_PAYMENT || 100000000000;


// Address of target node.
export const DEPLOY_NODE_ADDRESS =
  process.env.CSPR_INTS_DEPLOY_NODE_ADDRESS || "http://3.208.91.63:7777/rpc";

// Time interval in milliseconds after which deploy will not be processed by a node.
export const DEPLOY_TTL_MS = process.env.CSPR_INTS_DEPLOY_TTL_MS || 1800000;
