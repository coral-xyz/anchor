import camelCase from "camelcase";
import {
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  Idl,
  IdlSeed,
  IdlAccount,
  IdlAccountItem,
  IdlAccounts,
  IdlTypeDef,
  IdlTypeDefStruct,
  IdlTypeDefTyStruct,
  IdlType,
} from "../idl.js";
import * as utf8 from "../utils/bytes/utf8.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_PROGRAM_ID } from "../utils/token.js";
import { AllInstructions } from "./namespace/types.js";
import Provider from "../provider.js";
import { AccountNamespace } from "./namespace/account.js";
import { coder } from "../spl/token";
import { BorshAccountsCoder } from "src/coder/index.js";
import { Program } from "./index.js";

type Accounts = { [name: string]: PublicKey | Accounts };

export type CustomAccountResolver<IDL extends Idl> = (params: {
  args: Array<any>;
  accounts: Accounts;
  provider: Provider;
  programId: PublicKey;
  idlIx: AllInstructions<IDL>;
}) => Promise<{ accounts: Accounts; resolved: number }>;

// Populates a given accounts context with PDAs and common missing accounts.
export class AccountsResolver<IDL extends Idl, I extends AllInstructions<IDL>> {
  _args: Array<any>;
  static readonly CONST_ACCOUNTS = {
    associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
    rent: SYSVAR_RENT_PUBKEY,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    clock: SYSVAR_CLOCK_PUBKEY,
  };

  private _accountStore: AccountStore<IDL>;

  constructor(
    _args: Array<any>,
    private _accounts: Accounts,
    private _provider: Provider,
    private _programId: PublicKey,
    private _idlIx: AllInstructions<IDL>,
    _accountNamespace: AccountNamespace<IDL>,
    private _idlTypes: IdlTypeDef[],
    private _customResolver?: CustomAccountResolver<IDL>
  ) {
    this._args = _args;
    this._accountStore = new AccountStore(
      _provider,
      _accountNamespace,
      this._programId
    );
  }

  public args(_args: Array<any>): void {
    this._args = _args;
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
    await this.resolveConst(this._idlIx.accounts);

    // Auto populate pdas and relations until we stop finding new accounts
    while (
      (await this.resolvePdas(this._idlIx.accounts)) +
        (await this.resolveRelations(this._idlIx.accounts)) +
        (await this.resolveCustom()) >
      0
    ) {}
  }

  private async resolveCustom(): Promise<number> {
    if (this._customResolver) {
      const { accounts, resolved } = await this._customResolver({
        args: this._args,
        accounts: this._accounts,
        provider: this._provider,
        programId: this._programId,
        idlIx: this._idlIx,
      });
      this._accounts = accounts;
      return resolved;
    }
    return 0;
  }

  private get(path: string[]): PublicKey | undefined {
    // Only return if pubkey
    const ret = path.reduce(
      (acc, subPath) => acc && acc[subPath],
      this._accounts
    );

    if (ret && ret.toBase58) {
      return ret as PublicKey;
    }
  }

  private set(path: string[], value: PublicKey): void {
    let curr = this._accounts;
    path.forEach((p, idx) => {
      const isLast = idx == path.length - 1;
      if (isLast) {
        curr[p] = value;
      }

      curr[p] = curr[p] || {};
      curr = curr[p] as Accounts;
    });
  }

  private async resolveConst(
    accounts: IdlAccountItem[],
    path: string[] = []
  ): Promise<void> {
    for (let k = 0; k < accounts.length; k += 1) {
      const accountDescOrAccounts = accounts[k];
      const subAccounts = (accountDescOrAccounts as IdlAccounts).accounts;
      if (subAccounts) {
        await this.resolveConst(subAccounts, [
          ...path,
          camelCase(accountDescOrAccounts.name),
        ]);
      }

      const accountDesc = accountDescOrAccounts as IdlAccount;
      const accountDescName = camelCase(accountDescOrAccounts.name);

      // Signers default to the provider.
      if (accountDesc.isSigner && !this.get([...path, accountDescName])) {
        // @ts-expect-error
        if (this._provider.wallet === undefined) {
          throw new Error(
            "This function requires the Provider interface implementor to have a 'wallet' field."
          );
        }
        // @ts-expect-error
        this.set([...path, accountDescName], this._provider.wallet.publicKey);
      }

      // Common accounts are auto populated with magic names by convention.
      if (
        Reflect.has(AccountsResolver.CONST_ACCOUNTS, accountDescName) &&
        !this.get([...path, accountDescName])
      ) {
        this.set(
          [...path, accountDescName],
          AccountsResolver.CONST_ACCOUNTS[accountDescName]
        );
      }
    }
  }

  private async resolvePdas(
    accounts: IdlAccountItem[],
    path: string[] = []
  ): Promise<number> {
    let found = 0;
    for (let k = 0; k < accounts.length; k += 1) {
      const accountDesc = accounts[k];
      const subAccounts = (accountDesc as IdlAccounts).accounts;
      if (subAccounts) {
        found += await this.resolvePdas(subAccounts, [
          ...path,
          camelCase(accountDesc.name),
        ]);
      }

      const accountDescCasted: IdlAccount = accountDesc as IdlAccount;
      const accountDescName = camelCase(accountDesc.name);
      // PDA derived from IDL seeds.
      if (
        accountDescCasted.pda &&
        accountDescCasted.pda.seeds.length > 0 &&
        !this.get([...path, accountDescName])
      ) {
        if (Boolean(await this.autoPopulatePda(accountDescCasted, path))) {
          found += 1;
        }
      }
    }
    return found;
  }

  private async resolveRelations(
    accounts: IdlAccountItem[],
    path: string[] = []
  ): Promise<number> {
    let found = 0;
    for (let k = 0; k < accounts.length; k += 1) {
      const accountDesc = accounts[k];
      const subAccounts = (accountDesc as IdlAccounts).accounts;
      if (subAccounts) {
        found += await this.resolveRelations(subAccounts, [
          ...path,
          camelCase(accountDesc.name),
        ]);
      }
      const relations = (accountDesc as IdlAccount).relations || [];
      const accountDescName = camelCase(accountDesc.name);
      const newPath = [...path, accountDescName];

      // If we have this account and there's some missing accounts that are relations to this account, fetch them
      const accountKey = this.get(newPath);
      if (accountKey) {
        const matching = relations.filter(
          (rel) => !this.get([...path, camelCase(rel)])
        );

        found += matching.length;
        if (matching.length > 0) {
          const programId = await this.parseProgramId(
            accountDesc as IdlAccount
          );
          const account = await this._accountStore.fetchAccount({
            publicKey: accountKey,
            programId,
          });
          await Promise.all(
            matching.map(async (rel) => {
              const relName = camelCase(rel);

              this.set([...path, relName], account[relName]);
              return account[relName];
            })
          );
        }
      }
    }
    return found;
  }

  private async autoPopulatePda(accountDesc: IdlAccount, path: string[] = []) {
    if (!accountDesc.pda || !accountDesc.pda.seeds)
      throw new Error("Must have seeds");

    const seeds: (Buffer | undefined)[] = await Promise.all(
      accountDesc.pda.seeds.map((seedDesc: IdlSeed) => this.toBuffer(seedDesc))
    );

    if (seeds.some((seed) => typeof seed == "undefined")) {
      return;
    }

    const programId = await this.parseProgramId(accountDesc);
    if (!programId) {
      return;
    }
    const [pubkey] = await PublicKey.findProgramAddress(
      seeds as Buffer[],
      programId
    );

    this.set([...path, camelCase(accountDesc.name)], pubkey);
  }

  private async parseProgramId(accountDesc: IdlAccount): Promise<PublicKey> {
    if (!accountDesc.pda?.programId) {
      return this._programId;
    }
    switch (accountDesc.pda.programId.kind) {
      case "const":
        return new PublicKey(
          this.toBufferConst(accountDesc.pda.programId.value)
        );
      case "arg":
        return this.argValue(accountDesc.pda.programId);
      case "account":
        return await this.accountValue(accountDesc.pda.programId);
      default:
        throw new Error(
          `Unexpected program seed kind: ${accountDesc.pda.programId.kind}`
        );
    }
  }

  private async toBuffer(seedDesc: IdlSeed): Promise<Buffer | undefined> {
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

  /**
   * Recursively get the type at some path of either a primitive or a user defined struct.
   */
  private getType(type: IdlType, path: string[] = []): string {
    if (path.length > 0 && (type as any).defined) {
      const subType = this._idlTypes.find(
        (t) => t.name === (type as any).defined
      );
      if (!subType) {
        throw new Error(`Cannot find type ${(type as any).defined}`);
      }

      const structType = subType.type as IdlTypeDefTyStruct; // enum not supported yet
      const field = structType.fields.find((field) => field.name === path[0]);

      return this.getType(field!.type, path.slice(1));
    }

    return type as string;
  }

  private toBufferConst(seedDesc: IdlSeed): Buffer {
    return this.toBufferValue(
      this.getType(seedDesc.type, (seedDesc.path || "").split(".").slice(1)),
      seedDesc.value
    );
  }

  private async toBufferArg(seedDesc: IdlSeed): Promise<Buffer | undefined> {
    const argValue = this.argValue(seedDesc);
    if (!argValue) {
      return;
    }
    return this.toBufferValue(
      this.getType(seedDesc.type, (seedDesc.path || "").split(".").slice(1)),
      argValue
    );
  }

  private argValue(seedDesc: IdlSeed): any {
    const split = seedDesc.path.split(".");
    const seedArgName = camelCase(split[0]);

    const idlArgPosition = this._idlIx.args.findIndex(
      (argDesc: any) => argDesc.name === seedArgName
    );
    if (idlArgPosition === -1) {
      throw new Error(`Unable to find argument for seed: ${seedArgName}`);
    }

    return split
      .slice(1)
      .reduce((curr, path) => (curr || {})[path], this._args[idlArgPosition]);
  }

  private async toBufferAccount(
    seedDesc: IdlSeed
  ): Promise<Buffer | undefined> {
    const accountValue = await this.accountValue(seedDesc);
    if (!accountValue) {
      return;
    }
    return this.toBufferValue(seedDesc.type, accountValue);
  }

  private async accountValue(seedDesc: IdlSeed): Promise<any> {
    const pathComponents = seedDesc.path.split(".");

    const fieldName = pathComponents[0];
    const fieldPubkey = this._accounts[camelCase(fieldName)];

    // The seed is a pubkey of the account.
    if (pathComponents.length === 1) {
      return fieldPubkey;
    }

    // The key is account data.
    //
    // Fetch and deserialize it.
    const account = await this._accountStore.fetchAccount({
      publicKey: fieldPubkey as PublicKey,
      name: seedDesc.account,
    });

    // Dereference all fields in the path to get the field value
    // used in the seed.
    const fieldValue = this.parseAccountValue(account, pathComponents.slice(1));
    return fieldValue;
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
  private _idls: Record<string, AccountNamespace<any>> = {};

  // todo: don't use the progrma use the account namespace.
  constructor(
    private _provider: Provider,
    _accounts: AccountNamespace<IDL>,
    private _programId: PublicKey
  ) {
    this._idls[_programId.toBase58()] = _accounts;
  }

  private async ensureIdl(
    programId: PublicKey
  ): Promise<AccountNamespace<any> | undefined> {
    if (!this._idls[programId.toBase58()]) {
      const idl = await Program.fetchIdl(programId, this._provider);
      if (idl) {
        const program = new Program(idl, programId, this._provider);
        this._idls[programId.toBase58()] = program.account;
      }
    }

    return this._idls[programId.toBase58()];
  }

  public async fetchAccount<T = any>({
    publicKey,
    name,
    programId = this._programId,
  }: {
    publicKey: PublicKey;
    name?: string;
    programId?: PublicKey;
  }): Promise<T> {
    const address = publicKey.toString();
    if (!this._cache.has(address)) {
      if (name === "TokenAccount") {
        const accountInfo = await this._provider.connection.getAccountInfo(
          publicKey
        );
        if (accountInfo === null) {
          throw new Error(`invalid account info for ${address}`);
        }
        const data = coder().accounts.decode("token", accountInfo.data);
        this._cache.set(address, data);
      } else if (name) {
        const accounts = await this.ensureIdl(programId);
        if (accounts) {
          const accountFetcher = accounts[camelCase(name)];
          if (accountFetcher) {
            const account = await accountFetcher.fetch(publicKey);
            this._cache.set(address, account);
          }
        }
      } else {
        const account = await this._provider.connection.getAccountInfo(
          publicKey
        );
        if (account === null) {
          throw new Error(`invalid account info for ${address}`);
        }
        const data = account.data;
        const accounts = await this.ensureIdl(programId);
        if (accounts) {
          const firstAccountLayout = Object.values(accounts)[0] as any;
          if (!firstAccountLayout) {
            throw new Error("No accounts for this program");
          }

          const result = (
            firstAccountLayout.coder.accounts as BorshAccountsCoder
          ).decodeAny(data);
          this._cache.set(address, result);
        }
      }
    }
    return this._cache.get(address);
  }
}
