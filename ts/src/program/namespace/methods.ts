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
import { Idl, IdlAccount } from "../../idl.js";
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
    for (let k = 0; k < this._idlIx.accounts.length; k += 1) {
      // Cast is ok because only a non-nested IdlAccount can have a seeds
      // cosntraint.
      const accountDesc = this._idlIx.accounts[k] as IdlAccount;

      // Auto populate *if needed*.
      if (accountDesc.seeds && accountDesc.seeds.length > 0) {
        if (this._accounts[accountDesc.name] === undefined) {
          await this.autoPopulatePda(accountDesc);
        }
      }
    }
  }

  private async autoPopulatePda(accountDesc: IdlAccount) {
    const seeds = [];

    // todo

    const [pubkey] = await PublicKey.findProgramAddress(seeds, this._programId);

    this._accounts[accountDesc.name] = pubkey;
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
