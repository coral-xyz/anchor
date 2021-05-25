import EventEmitter from "eventemitter3";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionSignature,
  TransactionInstruction,
  SYSVAR_RENT_PUBKEY,
  Commitment,
} from "@solana/web3.js";
import Provider from "../../provider";
import { Idl, IdlStateMethod } from "../../idl";
import Coder, { stateDiscriminator } from "../../coder";
import { RpcNamespace, InstructionNamespace } from "./";
import {
  Subscription,
  translateError,
  toInstruction,
  validateAccounts,
} from "../common";
import { Accounts, splitArgsAndCtx } from "../context";
import InstructionNamespaceFactory from "./instruction";

export class StateClient {
  get rpc(): RpcNamespace {
    return this._rpc;
  }
  private _rpc: RpcNamespace;

  get instruction(): InstructionNamespace {
    return this._instruction;
  }
  private _instruction: InstructionNamespace;

  get programId(): PublicKey {
    return this._programId;
  }
  private _programId: PublicKey;

  get provider(): Provider {
    return this._provider;
  }
  private _provider: Provider;

  get coder(): Coder {
    return this._coder;
  }
  private _coder: Coder;

  private _idl: Idl;

  private _sub: Subscription | null;

  constructor(
    idl: Idl,
    coder: Coder,
    programId: PublicKey,
    idlErrors: Map<number, string>,
    provider: Provider
  ) {
    this._idl = idl;
    this._coder = coder;
    this._programId = programId;
    this._provider = provider;
    this._sub = null;
  }

  async address(): Promise<PublicKey> {
    return await programStateAddress(this.programId);
  }

  subscribe(commitment?: Commitment): EventEmitter {
    if (this._sub !== null) {
      return this._sub.ee;
    }
    const ee = new EventEmitter();

    this.address().then((address) => {
      const listener = this.provider.connection.onAccountChange(
        address,
        (acc) => {
          const account = this.coder.state.decode(acc.data);
          ee.emit("change", account);
        },
        commitment
      );

      this._sub = {
        ee,
        listener,
      };
    });

    return ee;
  }

  unsubscribe() {
    if (this._sub !== null) {
      this.provider.connection
        .removeAccountChangeListener(this._sub.listener)
        .then(async () => {
          this._sub = null;
        })
        .catch(console.error);
    }
  }
}

export type StateNamespace = () =>
  | Promise<any>
  | {
      address: () => Promise<PublicKey>;
      rpc: RpcNamespace;
      instruction: InstructionNamespace;
      subscribe: (commitment?: Commitment) => EventEmitter;
      unsubscribe: () => void;
    };

export default class StateFactory {
  // Builds the state namespace.
  public static build(
    idl: Idl,
    coder: Coder,
    programId: PublicKey,
    idlErrors: Map<number, string>,
    provider: Provider
  ): StateNamespace | undefined {
    if (idl.state === undefined) {
      return undefined;
    }

    // Fetches the state object from the blockchain.
    const state = async (): Promise<any> => {
      const addr = await programStateAddress(programId);
      const accountInfo = await provider.connection.getAccountInfo(addr);
      if (accountInfo === null) {
        throw new Error(`Account does not exist ${addr.toString()}`);
      }
      // Assert the account discriminator is correct.
      const expectedDiscriminator = await stateDiscriminator(
        idl.state.struct.name
      );
      if (expectedDiscriminator.compare(accountInfo.data.slice(0, 8))) {
        throw new Error("Invalid account discriminator");
      }
      return coder.state.decode(accountInfo.data);
    };

    // Namespace with all rpc functions.
    const rpc: RpcNamespace = {};
    const ix: InstructionNamespace = {};

    idl.state.methods.forEach((m: IdlStateMethod) => {
      const accounts = async (accounts: Accounts): Promise<any> => {
        const keys = await stateInstructionKeys(
          programId,
          provider,
          m,
          accounts
        );
        return keys.concat(
          InstructionNamespaceFactory.accountsArray(accounts, m.accounts)
        );
      };
      const ixFn = async (...args: any[]): Promise<TransactionInstruction> => {
        const [ixArgs, ctx] = splitArgsAndCtx(m, [...args]);
        return new TransactionInstruction({
          keys: await accounts(ctx.accounts),
          programId,
          data: coder.instruction.encodeState(
            m.name,
            toInstruction(m, ...ixArgs)
          ),
        });
      };
      ixFn["accounts"] = accounts;
      ix[m.name] = ixFn;

      rpc[m.name] = async (...args: any[]): Promise<TransactionSignature> => {
        const [, ctx] = splitArgsAndCtx(m, [...args]);
        const tx = new Transaction();
        if (ctx.instructions !== undefined) {
          tx.add(...ctx.instructions);
        }
        tx.add(await ix[m.name](...args));
        try {
          const txSig = await provider.send(tx, ctx.signers, ctx.options);
          return txSig;
        } catch (err) {
          let translatedErr = translateError(idlErrors, err);
          if (translatedErr === null) {
            throw err;
          }
          throw translatedErr;
        }
      };
    });

    state["rpc"] = rpc;
    state["instruction"] = ix;

    return state;
  }
}

// Calculates the deterministic address of the program's "state" account.
async function programStateAddress(programId: PublicKey): Promise<PublicKey> {
  let [registrySigner] = await PublicKey.findProgramAddress([], programId);
  return PublicKey.createWithSeed(registrySigner, "unversioned", programId);
}

// Returns the common keys that are prepended to all instructions targeting
// the "state" of a program.
async function stateInstructionKeys(
  programId: PublicKey,
  provider: Provider,
  m: IdlStateMethod,
  accounts: Accounts
) {
  if (m.name === "new") {
    // Ctor `new` method.
    const [programSigner] = await PublicKey.findProgramAddress([], programId);
    return [
      {
        pubkey: provider.wallet.publicKey,
        isWritable: false,
        isSigner: true,
      },
      {
        pubkey: await programStateAddress(programId),
        isWritable: true,
        isSigner: false,
      },
      { pubkey: programSigner, isWritable: false, isSigner: false },
      {
        pubkey: SystemProgram.programId,
        isWritable: false,
        isSigner: false,
      },

      { pubkey: programId, isWritable: false, isSigner: false },
      {
        pubkey: SYSVAR_RENT_PUBKEY,
        isWritable: false,
        isSigner: false,
      },
    ];
  } else {
    validateAccounts(m.accounts, accounts);
    return [
      {
        pubkey: await programStateAddress(programId),
        isWritable: true,
        isSigner: false,
      },
    ];
  }
}
