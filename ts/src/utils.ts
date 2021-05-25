import BN from "bn.js";
import * as bs58 from "bs58";
import { sha256 } from "crypto-hash";
import { sha256 as sha256Sync } from "js-sha256";
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

// Sync version of web3.PublicKey.createWithSeed.
export function createWithSeedSync(
  fromPublicKey: PublicKey,
  seed: string,
  programId: PublicKey
): PublicKey {
  const buffer = Buffer.concat([
    fromPublicKey.toBuffer(),
    Buffer.from(seed),
    programId.toBuffer(),
  ]);
  const hash = sha256Sync.digest(buffer);
  return new PublicKey(Buffer.from(hash));
}

// Sync version of web3.PublicKey.createProgramAddress.
export function createProgramAddressSync(
  seeds: Array<Buffer | Uint8Array>,
  programId: PublicKey
): PublicKey {
  const MAX_SEED_LENGTH = 32;

  let buffer = Buffer.alloc(0);
  seeds.forEach(function (seed) {
    if (seed.length > MAX_SEED_LENGTH) {
      throw new TypeError(`Max seed length exceeded`);
    }
    buffer = Buffer.concat([buffer, toBuffer(seed)]);
  });
  buffer = Buffer.concat([
    buffer,
    programId.toBuffer(),
    Buffer.from("ProgramDerivedAddress"),
  ]);
  let hash = sha256Sync(new Uint8Array(buffer));
  let publicKeyBytes = new BN(hash, 16).toArray(undefined, 32);
  if (PublicKey.isOnCurve(new Uint8Array(publicKeyBytes))) {
    throw new Error(`Invalid seeds, address must fall off the curve`);
  }
  return new PublicKey(publicKeyBytes);
}

// Sync version of web3.PublicKey.findProgramAddress.
export function findProgramAddressSync(
  seeds: Array<Buffer | Uint8Array>,
  programId: PublicKey
): [PublicKey, number] {
  let nonce = 255;
  let address: PublicKey | undefined;
  while (nonce != 0) {
    try {
      const seedsWithNonce = seeds.concat(Buffer.from([nonce]));
      address = createProgramAddressSync(seedsWithNonce, programId);
    } catch (err) {
      if (err instanceof TypeError) {
        throw err;
      }
      nonce--;
      continue;
    }
    return [address, nonce];
  }
  throw new Error(`Unable to find a viable program address nonce`);
}

const toBuffer = (arr: Buffer | Uint8Array | Array<number>): Buffer => {
  if (arr instanceof Buffer) {
    return arr;
  } else if (arr instanceof Uint8Array) {
    return Buffer.from(arr.buffer, arr.byteOffset, arr.byteLength);
  } else {
    return Buffer.from(arr);
  }
};

const utils = {
  bs58,
  sha256,
  getMultipleAccounts,
  idlAddress,
  publicKey: {
    createProgramAddressSync,
    findProgramAddressSync,
    createWithSeedSync,
  },
};

export default utils;
