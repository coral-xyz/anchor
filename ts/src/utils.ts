import * as bs58 from "bs58";
import { sha256 } from "crypto-hash";
import assert from "assert";
import { PublicKey, AccountInfo, Connection } from "@solana/web3.js";
import { idlAddress } from "./idl";

export const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

async function getMultipleAccounts(
  connection: Connection,
  publicKeys: PublicKey[]
): Promise<
  Array<null | { publicKey: PublicKey; account: AccountInfo<Buffer> }>
> {
  const args = [publicKeys.map((k) => k.toBase58()), { commitment: "recent" }];
  // @ts-ignore
  const res = await connection._rpcRequest("getMultipleAccounts", args);
  if (res.error) {
    throw new Error(
      "failed to get info about accounts " +
        publicKeys.map((k) => k.toBase58()).join(", ") +
        ": " +
        res.error.message
    );
  }
  assert(typeof res.result !== "undefined");
  const accounts: Array<null | {
    executable: any;
    owner: PublicKey;
    lamports: any;
    data: Buffer;
  }> = [];
  for (const account of res.result.value) {
    let value: {
      executable: any;
      owner: PublicKey;
      lamports: any;
      data: Buffer;
    } | null = null;
    if (account === null) {
      accounts.push(null);
      continue;
    }
    if (res.result.value) {
      const { executable, owner, lamports, data } = account;
      assert(data[1] === "base64");
      value = {
        executable,
        owner: new PublicKey(owner),
        lamports,
        data: Buffer.from(data[0], "base64"),
      };
    }
    if (value === null) {
      throw new Error("Invalid response");
    }
    accounts.push(value);
  }
  return accounts.map((account, idx) => {
    if (account === null) {
      return null;
    }
    return {
      publicKey: publicKeys[idx],
      account,
    };
  });
}

export function decodeUtf8(array: Uint8Array): string {
  const decoder =
    typeof TextDecoder === "undefined"
      ? new (require("util").TextDecoder)("utf-8") // Node.
      : new TextDecoder("utf-8"); // Browser.
  return decoder.decode(array);
}

const utils = {
  bs58,
  sha256,
  getMultipleAccounts,
  idlAddress,
};

export default utils;
