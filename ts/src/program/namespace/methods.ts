import camelCase from "camelcase";
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
import { SimulateResponse } from "./simulate.js";
import { TransactionFn } from "./transaction.js";
import { Idl, IdlSeed, IdlAccount } from "../../idl.js";
import * as utf8 from "../../utils/bytes/utf8.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_PROGRAM_ID } from "../../utils/token.js";
import { AllInstructions, MethodsFn, MakeMethodsNamespace } from "./types.js";
import { InstructionFn } from "./instruction.js";
import { RpcFn } from "./rpc.js";
import { SimulateFn } from "./simulate.js";
import Provider from "../../provider.js";
import { AccountNamespace } from "./account.js";
import { coder } from "../../spl/token";

export type MethodsNamespace<
  IDL extends Idl = Idl,
  I extends AllInstructions<IDL> = AllInstructions<IDL>
> = MakeMethodsNamespace<IDL, I, any>; // TODO: don't use any.

export class MethodsBuilderFactory {
  public static build<IDL extends Idl, I extends AllInstructions<IDL>>(
    provider: Provider,
    programId: PublicKey,
    idlIx: AllInstructions<IDL>,
    ixFn: InstructionFn<IDL>,
    txFn: TransactionFn<IDL>,
    rpcFn: RpcFn<IDL>,
    simulateFn: SimulateFn<IDL>,
    accountNamespace: AccountNamespace<IDL>
  ): MethodsFn<IDL, I, any> {
    const request: MethodsFn<IDL, I, any> = (...args) => {
      return new MethodsBuilder(
        args,
        ixFn,
        txFn,
        rpcFn,
        simulateFn,
        provider,
        programId,
        idlIx,
        accountNamespace
      );
    };
    return request;
  }
}

export class MethodsBuilder<IDL extends Idl, I extends AllInstructions<IDL>> {
  readonly _accounts: { [name: string]: PublicKey } = {};
  private _remainingAccounts: Array<AccountMeta> = [];
  private _signers: Array<Signer> = [];
  private _preInstructions: Array<TransactionInstruction> = [];
  private _postInstructions: Array<TransactionInstruction> = [];
  private _accountsResolver: AccountsResolver<IDL, I>;

  constructor(
    private _args: Array<any>,
    private _ixFn: InstructionFn<IDL>,
    private _txFn: TransactionFn<IDL>,
    private _rpcFn: RpcFn<IDL>,
    private _simulateFn: SimulateFn<IDL>,
    _provider: Provider,
    _programId: PublicKey,
    _idlIx: AllInstructions<IDL>,
    _accountNamespace: AccountNamespace<IDL>
  ) {
    this._accountsResolver = new AccountsResolver(
      _args,
      this._accounts,
      _provider,
      _programId,
      _idlIx,
      _accountNamespace
    );
  }

  // TODO: don't use any.
  public accounts(accounts: any): MethodsBuilder<IDL, I> {
    Object.assign(this._accounts, accounts);
    return this;
  }

  public signers(signers: Array<Signer>): MethodsBuilder<IDL, I> {
    this._signers = this._signers.concat(signers);
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
    await this._accountsResolver.resolve();
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
    await this._accountsResolver.resolve();
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
    await this._accountsResolver.resolve();
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
    await this._accountsResolver.resolve();
    // @ts-ignore
    return this._txFn(...this._args, {
      accounts: this._accounts,
      signers: this._signers,
      remainingAccounts: this._remainingAccounts,
      preInstructions: this._preInstructions,
      postInstructions: this._postInstructions,
    });
  }
}

class AccountsResolver<IDL extends Idl, I extends AllInstructions<IDL>> {
  private _accountStore: AccountStore<IDL>;

  constructor(
    private _args: Array<any>,
    private _accounts: { [name: string]: PublicKey },
    private _provider: Provider,
    private _programId: PublicKey,
    private _idlIx: AllInstructions<IDL>,
    _accountNamespace: AccountNamespace<IDL>
  ) {
    this._accountStore = new AccountStore(_provider, _accountNamespace);
  }

  // Note: We serially resolve PDAs one by one rather than doing them
  //       in parallel because there can be dependencies between
  //       addresses. That is, one PDA can be used as a seed in another.
  //
  // TODO: PDAs need to be resolved in topological order. For now, we
  //       require the developer to simply list the accounts in the
  //       correct order. But in future work, we should create the
  //       dependency graph and resolve automatically.
  //
  public async resolve() {
    for (let k = 0; k < this._idlIx.accounts.length; k += 1) {
      // Cast is ok because only a non-nested IdlAccount can have a seeds
      // cosntraint.
      const accountDesc = this._idlIx.accounts[k] as IdlAccount;
      const accountDescName = camelCase(accountDesc.name);

      // PDA derived from IDL seeds.
      if (accountDesc.pda && accountDesc.pda.seeds.length > 0) {
        if (this._accounts[accountDescName] === undefined) {
          await this.autoPopulatePda(accountDesc);
          continue;
        }
      }

      // Signers default to the provider.
      if (
        accountDesc.isSigner &&
        this._accounts[accountDescName] === undefined
      ) {
        this._accounts[accountDescName] = this._provider.wallet.publicKey;
        continue;
      }

      // Common accounts are auto populated with magic names by convention.
      switch (accountDescName) {
        case "systemProgram":
          if (this._accounts[accountDescName] === undefined) {
            this._accounts[accountDescName] = SystemProgram.programId;
          }
        case "rent":
          if (this._accounts[accountDescName] === undefined) {
            this._accounts[accountDescName] = SYSVAR_RENT_PUBKEY;
          }
        case "tokenProgram":
          if (this._accounts[accountDescName] === undefined) {
            this._accounts[accountDescName] = TOKEN_PROGRAM_ID;
          }
        case "associatedTokenProgram":
          if (this._accounts[accountDescName] === undefined) {
            this._accounts[accountDescName] = ASSOCIATED_PROGRAM_ID;
          }
      }
    }
  }

  private async autoPopulatePda(accountDesc: IdlAccount) {
    if (!accountDesc.pda || !accountDesc.pda.seeds)
      throw new Error("Must have seeds");

    const seeds: Buffer[] = await Promise.all(
      accountDesc.pda.seeds.map((seedDesc: IdlSeed) => this.toBuffer(seedDesc))
    );

    const programId = this.parseProgramId(accountDesc);
    const [pubkey] = await PublicKey.findProgramAddress(seeds, programId);

    this._accounts[camelCase(accountDesc.name)] = pubkey;
  }

  private parseProgramId(accountDesc: IdlAccount): PublicKey {
    // TODO
    return this._programId;
  }

  private async toBuffer(seedDesc: IdlSeed): Promise<Buffer> {
    switch (seedDesc.kind) {
      case "const":
        return this.toBufferConst(seedDesc);
      case "arg":
        return await this.toBufferArg(seedDesc);
      case "account":
        return await this.toBufferAccount(seedDesc);
      default:
        throw new Error(`Unexpected seed kind: ${seedDesc.kind}`);
    }
  }

  private toBufferConst(seedDesc: IdlSeed): Buffer {
    return this.toBufferValue(seedDesc.type, seedDesc.value);
  }

  private async toBufferArg(seedDesc: IdlSeed): Promise<Buffer> {
    const seedArgName = camelCase(seedDesc.path.split(".")[0]);

    const idlArgPosition = this._idlIx.args.findIndex(
      (argDesc: any) => argDesc.name === seedArgName
    );
    if (idlArgPosition === -1) {
      throw new Error(`Unable to find argument for seed: ${seedArgName}`);
    }

    const argValue = this._args[idlArgPosition];
    return this.toBufferValue(seedDesc.type, argValue);
  }

  private async toBufferAccount(seedDesc: IdlSeed): Promise<Buffer> {
    const pathComponents = seedDesc.path.split(".");

    const fieldName = pathComponents[0];
    const fieldPubkey = this._accounts[camelCase(fieldName)];

    // The seed is a pubkey of the account.
    if (pathComponents.length === 1) {
      return this.toBufferValue("publicKey", fieldPubkey);
    }

    // The key is account data.
    //
    // Fetch and deserialize it.
    const account = await this._accountStore.fetchAccount(
      seedDesc.account,
      fieldPubkey
    );

    // Dereference all fields in the path to get the field value
    // used in the seed.
    const fieldValue = this.parseAccountValue(account, pathComponents.slice(1));

    // Now that we have the seed value, convert it into a buffer.
    return this.toBufferValue(seedDesc.type, fieldValue);
  }

  private parseAccountValue<T = any>(account: T, path: Array<string>): any {
    let accountField: any;
    while (path.length > 0) {
      accountField = account[camelCase(path[0])];
      path = path.slice(1);
    }
    return accountField;
  }

  // Converts the given idl valaue into a Buffer. The values here must be
  // primitives. E.g. no structs.
  //
  // TODO: add more types here as needed.
  private toBufferValue(type: string | any, value: any): Buffer {
    switch (type) {
      case "u8":
        return Buffer.from([value]);
      case "u16":
        let b = Buffer.alloc(2);
        b.writeUInt16LE(value);
        return b;
      case "u32":
        let buf = Buffer.alloc(4);
        buf.writeUInt32LE(value);
        return buf;
      case "u64":
        let bU64 = Buffer.alloc(8);
        bU64.writeBigUInt64LE(BigInt(value));
        return bU64;
      case "string":
        return Buffer.from(utf8.encode(value));
      case "publicKey":
        return value.toBuffer();
      default:
        if (type.array) {
          return Buffer.from(value);
        }
        throw new Error(`Unexpected seed type: ${type}`);
    }
  }
}

// TODO: this should be configureable to avoid unnecessary requests.
export class AccountStore<IDL extends Idl> {
  private _cache = new Map<string, any>();

  // todo: don't use the progrma use the account namespace.
  constructor(
    private _provider: Provider,
    private _accounts: AccountNamespace<IDL>
  ) {}

  public async fetchAccount<T = any>(
    name: string,
    publicKey: PublicKey
  ): Promise<T> {
    const address = publicKey.toString();
    if (this._cache.get(address) === undefined) {
      if (name === "TokenAccount") {
        const accountInfo = await this._provider.connection.getAccountInfo(
          publicKey
        );
        if (accountInfo === null) {
          throw new Error(`invalid account info for ${address}`);
        }
        const data = coder().accounts.decode("Token", accountInfo.data);
        this._cache.set(address, data);
      } else {
        const account = this._accounts[camelCase(name)].fetch(publicKey);
        this._cache.set(address, account);
      }
    }
    return this._cache.get(address);
  }
}
