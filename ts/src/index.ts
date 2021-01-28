import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import Provider from './provider';
import { Program } from "./program";
import Coder from "./coder";
import { Idl } from "./idl";
import workspace from "./workspace";
import utils from "./utils";

let _provider: Provider | null = null;

function setProvider(provider: Provider) {
  _provider = provider;
}

function getProvider(): Provider {
  if (_provider === null) {
    return Provider.local();
  }
  return _provider;
}

export {
  workspace,
  Program,
  Coder,
  setProvider,
  getProvider,
  Provider,
  BN,
  web3,
  Idl,
  utils,
};
