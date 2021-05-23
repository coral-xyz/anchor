import camelCase from "camelcase";
import EventEmitter from "eventemitter3";
import * as bs58 from "bs58";
import {
  Signer,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  Commitment,
} from "@solana/web3.js";
import Provider from "../../provider";
import { Idl } from "../../idl";
import Coder, {
  ACCOUNT_DISCRIMINATOR_SIZE,
  accountDiscriminator,
  accountSize,
} from "../../coder";
import { Subscription, Address, translateAddress } from "../common";

/**
 * Accounts is a dynamically generated object to fetch any given account
 * of a program.
 */
export interface AccountNamespace {
  [key: string]: AccountFn;
}

/**
 * Account is a function returning a deserialized account, given an address.
 */
export type AccountFn<T = any> = AccountProps & ((address: PublicKey) => T);

/**
 * Non function properties on the acccount namespace.
 */
type AccountProps = {
  size: number;
  all: (filter?: Buffer) => Promise<ProgramAccount<any>[]>;
  subscribe: (address: Address, commitment?: Commitment) => EventEmitter;
  unsubscribe: (address: Address) => void;
  createInstruction: (signer: Signer) => Promise<TransactionInstruction>;
  associated: (...args: PublicKey[]) => Promise<any>;
  associatedAddress: (...args: PublicKey[]) => Promise<PublicKey>;
};

/**
 * @hidden
 *
 * Deserialized account owned by a program.
 */
export type ProgramAccount<T = any> = {
  publicKey: PublicKey;
  account: T;
};

// Tracks all subscriptions.
const subscriptions: Map<string, Subscription> = new Map();

export default class AccountFactory {
  // Returns the generated accounts namespace.
  public static build(
    idl: Idl,
    coder: Coder,
    programId: PublicKey,
    provider: Provider
  ): AccountNamespace {
    const accountFns: AccountNamespace = {};

    idl.accounts.forEach((idlAccount) => {
      const name = camelCase(idlAccount.name);

      // Fetches the decoded account from the network.
      const accountsNamespace = async (address: Address): Promise<any> => {
        const accountInfo = await provider.connection.getAccountInfo(
          translateAddress(address)
        );
        if (accountInfo === null) {
          throw new Error(`Account does not exist ${address.toString()}`);
        }

        // Assert the account discriminator is correct.
        const discriminator = await accountDiscriminator(idlAccount.name);
        if (discriminator.compare(accountInfo.data.slice(0, 8))) {
          throw new Error("Invalid account discriminator");
        }

        return coder.accounts.decode(idlAccount.name, accountInfo.data);
      };

      // Returns the size of the account.
      // @ts-ignore
      accountsNamespace["size"] =
        ACCOUNT_DISCRIMINATOR_SIZE + accountSize(idl, idlAccount);

      // Returns an instruction for creating this account.
      // @ts-ignore
      accountsNamespace["createInstruction"] = async (
        signer: Signer,
        sizeOverride?: number
      ): Promise<TransactionInstruction> => {
        // @ts-ignore
        const size = accountsNamespace["size"];

        return SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: signer.publicKey,
          space: sizeOverride ?? size,
          lamports: await provider.connection.getMinimumBalanceForRentExemption(
            sizeOverride ?? size
          ),
          programId,
        });
      };

      // Subscribes to all changes to this account.
      // @ts-ignore
      accountsNamespace["subscribe"] = (
        address: Address,
        commitment?: Commitment
      ): EventEmitter => {
        if (subscriptions.get(address.toString())) {
          return subscriptions.get(address.toString()).ee;
        }

        const ee = new EventEmitter();
        address = translateAddress(address);
        const listener = provider.connection.onAccountChange(
          address,
          (acc) => {
            const account = coder.accounts.decode(idlAccount.name, acc.data);
            ee.emit("change", account);
          },
          commitment
        );

        subscriptions.set(address.toString(), {
          ee,
          listener,
        });

        return ee;
      };

      // Unsubscribes to account changes.
      // @ts-ignore
      accountsNamespace["unsubscribe"] = (address: Address) => {
        let sub = subscriptions.get(address.toString());
        if (!sub) {
          console.warn("Address is not subscribed");
          return;
        }
        if (subscriptions) {
          provider.connection
            .removeAccountChangeListener(sub.listener)
            .then(() => {
              subscriptions.delete(address.toString());
            })
            .catch(console.error);
        }
      };

      // Returns all instances of this account type for the program.
      // @ts-ignore
      accountsNamespace["all"] = async (
        filter?: Buffer
      ): Promise<ProgramAccount<any>[]> => {
        let bytes = await accountDiscriminator(idlAccount.name);
        if (filter !== undefined) {
          bytes = Buffer.concat([bytes, filter]);
        }
        // @ts-ignore
        let resp = await provider.connection._rpcRequest("getProgramAccounts", [
          programId.toBase58(),
          {
            commitment: provider.connection.commitment,
            filters: [
              {
                memcmp: {
                  offset: 0,
                  bytes: bs58.encode(bytes),
                },
              },
            ],
          },
        ]);
        if (resp.error) {
          console.error(resp);
          throw new Error("Failed to get accounts");
        }
        return (
          resp.result
            // @ts-ignore
            .map(({ pubkey, account: { data } }) => {
              data = bs58.decode(data);
              return {
                publicKey: new PublicKey(pubkey),
                account: coder.accounts.decode(idlAccount.name, data),
              };
            })
        );
      };

      // Function returning the associated address. Args are keys to associate.
      // Order matters.
      accountsNamespace["associatedAddress"] = async (
        ...args: Address[]
      ): Promise<PublicKey> => {
        let seeds = [Buffer.from([97, 110, 99, 104, 111, 114])]; // b"anchor".
        args.forEach((arg) => {
          seeds.push(translateAddress(arg).toBuffer());
        });
        const [assoc] = await PublicKey.findProgramAddress(seeds, programId);
        return assoc;
      };

      // Function returning the associated account. Args are keys to associate.
      // Order matters.
      accountsNamespace["associated"] = async (
        ...args: Address[]
      ): Promise<any> => {
        const addr = await accountsNamespace["associatedAddress"](...args);
        return await accountsNamespace(addr);
      };

      accountFns[name] = accountsNamespace;
    });

    return accountFns;
  }
}
