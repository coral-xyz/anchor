import {
  ConfirmOptions,
  AccountMeta,
  Signer,
  Transaction,
  TransactionInstruction,
  TransactionSignature,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { SimulateResponse } from "./simulate";
import { TransactionFn } from "./transaction.js";
import { Idl, IdlSeed, IdlAccount } from "../../idl.js";
import * as utf8 from "../../utils/bytes/utf8";
import {
  AllInstructions,
  InstructionContextFn,
  MakeInstructionsNamespace,
} from "./types";
import { InstructionFn } from "./instruction";
import { RpcFn } from "./rpc";
import { SimulateFn } from "./simulate";

export class MethodsBuilderFactory {
  public static build<IDL extends Idl, I extends AllInstructions<IDL>>(
    programId: PublicKey,
    idl: IDL,
    idlIx: AllInstructions<IDL>,
    ixFn: InstructionFn<IDL>,
    txFn: TransactionFn<IDL>,
    rpcFn: RpcFn<IDL>,
    simulateFn: SimulateFn<IDL>
  ): MethodFn {
    const request: MethodFn<IDL, I> = (...args) => {
      return new MethodsBuilder(
        programId,
        idl,
        idlIx,
        args,
        ixFn,
        txFn,
        rpcFn,
        simulateFn
      );
    };
    return request;
  }
}

export class MethodsBuilder<IDL extends Idl, I extends AllInstructions<IDL>> {
  private _accounts: { [name: string]: PublicKey } = {};
  private _remainingAccounts: Array<AccountMeta> = [];
  private _signers: Array<Signer> = [];
  private _preInstructions: Array<TransactionInstruction> = [];
  private _postInstructions: Array<TransactionInstruction> = [];

  constructor(
    private _programId: PublicKey,
    private _idl: IDL,
    private _idlIx: AllInstructions<IDL>,
    private _args: Array<any>,
    private _ixFn: InstructionFn<IDL>,
    private _txFn: TransactionFn<IDL>,
    private _rpcFn: RpcFn<IDL>,
    private _simulateFn: SimulateFn<IDL>
  ) {}

  // TODO: don't use any.
  public accounts(accounts: any): MethodsBuilder<IDL, I> {
    Object.assign(this._accounts, accounts);
    return this;
  }

  public signers(signers: Array<Signer>): MethodsBuilder<IDL, I> {
    Object.assign(this._signers, signers);
    return this;
  }

  public remainingAccounts(
    accounts: Array<AccountMeta>
  ): MethodsBuilder<IDL, I> {
    this._remainingAccounts = this._remainingAccounts.concat(accounts);
    return this;
  }

  public preInstructions(
    ixs: Array<TransactionInstruction>
  ): MethodsBuilder<IDL, I> {
    this._preInstructions = this._preInstructions.concat(ixs);
    return this;
  }

  public postInstructions(
    ixs: Array<TransactionInstruction>
  ): MethodsBuilder<IDL, I> {
    this._postInstructions = this._postInstructions.concat(ixs);
    return this;
  }

  public async rpc(options: ConfirmOptions): Promise<TransactionSignature> {
    await this.resolvePdas();
    // @ts-ignore
    return this._rpcFn(...this._args, {
      accounts: this._accounts,
      signers: this._signers,
      remainingAccounts: this._remainingAccounts,
      preInstructions: this._preInstructions,
      postInstructions: this._postInstructions,
      options: options,
    });
  }

  public async simulate(
    options: ConfirmOptions
  ): Promise<SimulateResponse<any, any>> {
    await this.resolvePdas();
    // @ts-ignore
    return this._simulateFn(...this._args, {
      accounts: this._accounts,
      signers: this._signers,
      remainingAccounts: this._remainingAccounts,
      preInstructions: this._preInstructions,
      postInstructions: this._postInstructions,
      options: options,
    });
  }

  public async instruction(): Promise<TransactionInstruction> {
    await this.resolvePdas();
    // @ts-ignore
    return this._ixFn(...this._args, {
      accounts: this._accounts,
      signers: this._signers,
      remainingAccounts: this._remainingAccounts,
      preInstructions: this._preInstructions,
      postInstructions: this._postInstructions,
    });
  }

  public async transaction(): Promise<Transaction> {
    await this.resolvePdas();
    // @ts-ignore
    return this._txFn(...this._args, {
      accounts: this._accounts,
      signers: this._signers,
      remainingAccounts: this._remainingAccounts,
      preInstructions: this._preInstructions,
      postInstructions: this._postInstructions,
    });
  }

  private async resolvePdas() {
    const promises: Array<Promise<any>> = [];
    for (let k = 0; k < this._idlIx.accounts.length; k += 1) {
      // Cast is ok because only a non-nested IdlAccount can have a seeds
      // cosntraint.
      const accountDesc = this._idlIx.accounts[k] as IdlAccount;

      // Auto populate *if needed*.
      if (accountDesc.seeds && accountDesc.seeds.length > 0) {
        if (this._accounts[accountDesc.name] === undefined) {
          promises.push(this.autoPopulatePda(accountDesc));
        }
      }
    }
    await Promise.all(promises);
  }

  private async autoPopulatePda(accountDesc: IdlAccount) {
    if (!accountDesc.seeds) throw new Error("Must have seeds");
    const seeds: Buffer[] = [];

    for (let k = 0; k < accountDesc.seeds.length; k += 1) {
      let seedDesc = accountDesc.seeds[k];
      seeds.push(this.toBuffer(seedDesc));
    }

    const [pubkey] = await PublicKey.findProgramAddress(seeds, this._programId);

    this._accounts[accountDesc.name] = pubkey;
  }

  private toBuffer(seedDesc: IdlSeed): Buffer {
    switch (seedDesc.kind) {
      case "const":
        return this.toBufferConst(seedDesc);
      case "arg":
        return this.toBufferArg(seedDesc);
      case "account":
        return this.toBufferAccount(seedDesc);
      default:
        throw new Error(`Unexpected seed kind: ${seedDesc.kind}`);
    }
  }

  private toBufferConst(seedDesc: IdlSeed): Buffer {
    return this.toBufferValue(seedDesc.type, seedDesc.value);
  }

  private toBufferArg(seedDesc: IdlSeed): Buffer {
    let idlArgPosition = -1;
    for (let k = 0; k < this._idlIx.args.length; k += 1) {
      const argDesc = this._idlIx.args[k];
      if (argDesc.name === seedDesc.name) {
        idlArgPosition = k;
        break;
      }
    }
    if (idlArgPosition === -1) {
      throw new Error(`Unable to find argument for seed: ${seedDesc.name}`);
    }

    const argValue = this._args[idlArgPosition];
    return this.toBufferValue(seedDesc.type, argValue);
  }

  private toBufferAccount(seedDesc: IdlSeed): Buffer {
    // 1. get the value
    // 2. convert the value into a buffer based on type
    return Buffer.from([]);
  }

  // Converts the given idl valaue into a Buffer. The values here must be
  // primitives. E.g. no structs.
  private toBufferValue(type: string | any, value: any): Buffer {
    switch (type) {
      case "u8":
        return Buffer.from([value]);
      case "u16":
        // todo
        return Buffer.from([]);
      case "u32":
        // todo
        return Buffer.from([]);
      case "u64":
        // todo
        return Buffer.from([]);
      case "string":
        return Buffer.from(utf8.encode(value));
      default:
        if (type.array) {
          return Buffer.from(value);
        }
        throw new Error(`Unexpected seed type: ${type}`);
    }
  }
}

export type MethodsNamespace<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = MakeInstructionsNamespace<IDL, I, any>; // TODO: don't use any.

export type MethodFn<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = InstructionContextFn<IDL, I, MethodsBuilder<IDL, I>>;
