import { Program } from "./program/index.js";
import { isBrowser } from "./utils/common.js";

export { default as BN } from "bn.js";
export * as web3 from "@solana/web3.js";
export { default as NodeWallet } from "./nodewallet";
export {
  default as Provider,
  getProvider,
  setProvider,
  AnchorProvider,
  Wallet
} from "./provider.js";
export * from "./error.js";
export { Instruction } from "./coder/borsh/instruction.js";
export { Idl } from "./idl.js";
export * from "./coder/index.js";
export * as utils from "./utils/index.js";
export * from "./program/index.js";
export * from "./spl/index.js";

export declare const workspace: Record<string, Program>;

if (!isBrowser) {
  exports.workspace = require("./workspace.js").default;
  exports.Wallet = require("./nodewallet.js").default;
}
