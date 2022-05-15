import { PublicKey } from "@solana/web3.js";
import { Program } from "../program/index.js";
import Provider from "../provider.js";
import { DexCoder } from "../coder/dex/index.js";

const DEX_PROGRAM_ID = new PublicKey(
  // TODO
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

export function program(provider?: Provider): Program<Dex> {
  return new Program<Dex>(IDL, DEX_PROGRAM_ID, provider, new DexCoder(IDL));
}

// TODO
export type Dex = {
  version: "0.1.0";
  name: "spl_token";
  instructions: [];
  accounts: [];
};

// TODO
export const IDL: Dex = {
  version: "0.1.0",
  name: "spl_token",
  instructions: [],
  accounts: [],
};
