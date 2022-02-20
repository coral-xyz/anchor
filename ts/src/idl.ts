import camelCase from "camelcase";
import { Buffer } from "buffer";
import { PublicKey } from "@solana/web3.js";
import * as borsh from "@project-serum/borsh";
import camelcase from "camelcase";
import { size } from "lodash";

export type RawIdl = {
  version: string;
  name: string;
  instructions: RawIdlInstruction[];
  state?: RawIdlState;
  accounts?: RawIdlTypeDef[];
  types?: RawIdlTypeDef[];
  events?: RawIdlEvent[];
  errors?: IdlErrorCode[];
  constants?: RawIdlConstant[];
};

export type Idl = {
  version: string;
  name: string;
  instructions: IdlInstruction[];
  state?: IdlState;
  accounts?: IdlTypeDef[];
  types?: IdlTypeDef[];
  events?: IdlEvent[];
  errors?: IdlErrorCode[];
  constants?: IdlConstant[];
  camelized?: boolean;
  metadata?: IdlMetadata;
};

type RawIdlEvent = {
  name: string;
  fields: RawIdlEventField[];
};

export type IdlEvent = {
  name: string;
  fields: IdlEventField[];
};

export type RawIdlConstant = {
  name: string;
  type: RawIdlType;
  value: string;

};

export type IdlMetadata = any;

export type IdlConstant = {
  name: string;
  type: IdlType;
  value: string;
};

type RawIdlEventField = {
  name: string;
  type: RawIdlType;
  index: boolean;
};

export type IdlEventField = {
  name: string;
  type: IdlType;
  index: boolean;
};

type RawIdlInstruction = {
  name: string;
  accounts: RawIdlAccountItem[];
  args: RawIdlField[];
};

export type IdlInstruction = {
  name: string;
  accounts: IdlAccountItem[];
  args: IdlField[];
};

type RawIdlState = {
  struct: RawIdlTypeDef;
  methods: RawIdlStateMethod[];
};

export type IdlState = {
  struct: IdlTypeDef;
  methods: IdlStateMethod[];
};

type RawIdlStateMethod = RawIdlInstruction;

export type IdlStateMethod = IdlInstruction;

type RawIdlAccountItem = RawIdlAccount | RawIdlAccounts;

export type IdlAccountItem = IdlAccount | IdlAccounts;

type RawIdlAccount = {
  name: string;
  is_mut: boolean;
  is_signer: boolean;
  pda?: RawIdlPda;
};

export type IdlAccount = {
  name: string;
  isMut: boolean;
  isSigner: boolean;
  pda?: IdlPda;
};

// A nested/recursive version of RawIdlAccount.
type RawIdlAccounts = {
  name: string;
  accounts: RawIdlAccountItem[];
};

type RawIdlPda = {
  seeds: IdlSeed[];
  program_id?: IdlSeed;
};

export type IdlPda = {
  seeds: IdlSeed[];
  programId?: IdlSeed;
};

export type IdlSeed = any; // TODO

// A nested/recursive version of IdlAccount.
export type IdlAccounts = {
  name: string;
  accounts: IdlAccountItem[];
};

type RawIdlField = {
  name: string;
  type: RawIdlType;
};

export type IdlField = {
  name: string;
  type: IdlType;
};

type RawIdlTypeDef = {
  name: string;
  type: RawIdlTypeDefTy;
};

export type IdlTypeDef = {
  name: string;
  type: IdlTypeDefTy;
};

type RawIdlTypeDefTyStruct = {
  kind: "struct";
  fields: RawIdlTypeDefStruct;
};

export type IdlTypeDefTyStruct = {
  kind: "struct";
  fields: IdlTypeDefStruct;
};

type RawIdlTypeDefTyEnum = {
  kind: "enum";
  variants: RawIdlEnumVariant[];
};

export type IdlTypeDefTyEnum = {
  kind: "enum";
  variants: IdlEnumVariant[];
};

type RawIdlTypeDefTy = RawIdlTypeDefTyEnum | RawIdlTypeDefTyStruct;

type IdlTypeDefTy = IdlTypeDefTyEnum | IdlTypeDefTyStruct;

type RawIdlTypeDefStruct = Array<RawIdlField>;

type IdlTypeDefStruct = Array<IdlField>;

type RawLiteralIdlType =
  | "Bool"
  | "U8"
  | "I8"
  | "U16"
  | "I16"
  | "U32"
  | "I32"
  | "U64"
  | "I64"
  | "U128"
  | "I128"
  | "Bytes"
  | "String"
  | "PublicKey";

type RawNonLiteralIdlType =
  | RawIdlTypeDefined
  | RawIdlTypeOption
  | RawIdlTypeVec
  | RawIdlTypeArray;

type RawIdlType = RawLiteralIdlType | RawNonLiteralIdlType;

export type LiteralIdlType =
  | "bool"
  | "u8"
  | "i8"
  | "u16"
  | "i16"
  | "u32"
  | "i32"
  | "f32"
  | "u64"
  | "i64"
  | "f64"
  | "u128"
  | "i128"
  | "bytes"
  | "string"
  | "publicKey";

export type NonLiteralIdlType =
  | IdlTypeDefined
  | IdlTypeOption
  | IdlTypeCOption
  | IdlTypeVec
  | IdlTypeArray;

export type IdlType = LiteralIdlType | NonLiteralIdlType;

// User defined type.
type RawIdlTypeDefined = {
  Defined: string;
};

// User defined type.
export type IdlTypeDefined = {
  defined: string;
};

type RawIdlTypeOption = {
  Option: RawIdlType;
};

export type IdlTypeOption = {
  option: IdlType;
};

type RawIdlTypeVec = {
  Vec: RawIdlType;
};

export type IdlTypeCOption = {
  coption: IdlType;
};

export type IdlTypeVec = {
  vec: IdlType;
};

type RawIdlTypeArray = {
  Array: [idlType: RawIdlType, size: number];
};

export type IdlTypeArray = {
  array: [idlType: IdlType, size: number];
};

type RawIdlEnumVariant = {
  name: string;
  fields?: RawIdlEnumFields;
};

export type IdlEnumVariant = {
  name: string;
  fields?: IdlEnumFields;
};

type RawIdlEnumFields = RawIdlEnumFieldsNamed | RawIdlEnumFieldsTuple;

type IdlEnumFields = IdlEnumFieldsNamed | IdlEnumFieldsTuple;

type RawIdlEnumFieldsNamed = RawIdlField[];

type IdlEnumFieldsNamed = IdlField[];

type RawIdlEnumFieldsTuple = RawIdlType[];

type IdlEnumFieldsTuple = IdlType[];

export type IdlErrorCode = {
  code: number;
  name: string;
  msg?: string;
};

function camelCaseAccount(acc: RawIdlAccount): IdlAccount {
  return {
    name: camelcase(acc.name),
    isMut: acc.is_mut,
    isSigner: acc.is_signer,
  };
}

function isAccounts(account: RawIdlAccountItem): account is RawIdlAccounts {
  return (account as RawIdlAccounts).accounts !== undefined;
}

function camelCaseAccountItem(account: RawIdlAccount): IdlAccount;
function camelCaseAccountItem(account: RawIdlAccounts): IdlAccounts;
function camelCaseAccountItem(account: RawIdlAccountItem): IdlAccountItem {
  if (isAccounts(account)) {
    return {
      name: camelCase(account.name),
      accounts: camelCaseAccountItems(account.accounts),
    };
  }
  return camelCaseAccount(account);
}

function camelCaseAccountItems(
  accounts: RawIdlAccountItem[]
): IdlAccountItem[] {
  return accounts.map(camelCaseAccountItem);
}

function isLiteral(type: RawIdlType): type is RawLiteralIdlType {
  return typeof (type as RawLiteralIdlType) === "string";
}

function isIdlTypeDefined(
  type: RawNonLiteralIdlType
): type is RawIdlTypeDefined {
  return (type as RawIdlTypeDefined).Defined !== undefined;
}

function isIdlTypeOption(type: RawNonLiteralIdlType): type is RawIdlTypeOption {
  return (type as RawIdlTypeOption).Option !== undefined;
}

function isIdlTypeVec(type: RawNonLiteralIdlType): type is RawIdlTypeVec {
  return (type as RawIdlTypeVec).Vec !== undefined;
}

function camelCaseIdlTypeDefined(type: RawIdlTypeDefined): IdlTypeDefined {
  return { defined: type.Defined };
}

function camelCaseIdlTypeOption(type: RawIdlTypeOption): IdlTypeOption {
  return { option: camelCaseIdlType(type.Option) };
}

function camelCaseIdlTypeVec(type: RawIdlTypeVec): IdlTypeVec {
  return { vec: camelCaseIdlType(type.Vec) };
}

function camelCaseIdlTypeArray(type: RawIdlTypeArray): IdlTypeArray {
  return { array: [camelCaseIdlType(type.Array[0]), type.Array[1]] };
}

function camelCaseIdlType(type: RawIdlType): IdlType {
  if (isLiteral(type)) {
    return camelCase(type) as LiteralIdlType;
  }
  if (isIdlTypeDefined(type)) {
    return camelCaseIdlTypeDefined(type);
  }
  if (isIdlTypeOption(type)) {
    return camelCaseIdlTypeOption(type);
  }
  if (isIdlTypeVec(type)) {
    return camelCaseIdlTypeVec(type);
  }
  return camelCaseIdlTypeArray(type);
}

function camelCaseIdlField(field: RawIdlField): IdlField {
  return {
    name: camelCase(field.name),
    type: camelCaseIdlType(field.type),
  };
}

function camelCaseIdlFields(args: RawIdlField[]): IdlField[] {
  return args.map(camelCaseIdlField);
}

function camelCaseIdlInstruction(
  rawInstruction: RawIdlInstruction
): IdlInstruction {
  return {
    name: camelcase(rawInstruction.name),
    accounts: camelCaseAccountItems(rawInstruction.accounts),
    args: camelCaseIdlFields(rawInstruction.args),
  };
}

function camelCaseIdlInstructions(
  rawIdlInstructions: RawIdlInstruction[]
): IdlInstruction[] {
  return rawIdlInstructions.map(camelCaseIdlInstruction);
}

function isRawIdlTypeDefTyEnum(
  rawIdlTypeDefTy: RawIdlTypeDefTy
): rawIdlTypeDefTy is RawIdlTypeDefTyEnum {
  return rawIdlTypeDefTy.kind === "enum";
}

function isRawIdlEnumFieldsNamed(
  fields: RawIdlEnumFields
): fields is RawIdlEnumFieldsNamed {
  return (fields as RawIdlEnumFieldsNamed)[0].name !== undefined;
}

function camelCaseIdlEnumFieldsNamed(
  fields: RawIdlEnumFieldsNamed
): IdlEnumFieldsNamed {
  return fields.map(camelCaseIdlField);
}

function camelCaseIdlEnumFieldsTuple(
  fields: RawIdlEnumFieldsTuple
): IdlEnumFieldsTuple {
  return fields.map(camelCaseIdlType);
}

function camelCaseIdlEnumFields(
  fields?: RawIdlEnumFields
): IdlEnumFields | undefined {
  if (fields === undefined) {
    return undefined;
  }
  if (isRawIdlEnumFieldsNamed(fields)) {
    return camelCaseIdlEnumFieldsNamed(fields);
  }
  return camelCaseIdlEnumFieldsTuple(fields);
}

function camelCaseIdlEnumVariant(variant: RawIdlEnumVariant): IdlEnumVariant {
  return {
    name: camelCase(variant.name),
    fields: camelCaseIdlEnumFields(variant.fields),
  };
}

function camelCaseIdlEnumVariants(
  variants: RawIdlEnumVariant[]
): IdlEnumVariant[] {
  return variants.map(camelCaseIdlEnumVariant);
}

function camelCaseIdlTypeDefStruct(
  fields: RawIdlTypeDefStruct
): IdlTypeDefStruct {
  return fields.map(camelCaseIdlField);
}

function camelCaseIdlTypeDefTy(rawIdlTypeDefTy: RawIdlTypeDefTy): IdlTypeDefTy {
  if (isRawIdlTypeDefTyEnum(rawIdlTypeDefTy)) {
    return {
      kind: "enum",
      variants: camelCaseIdlEnumVariants(rawIdlTypeDefTy.variants),
    };
  }
  return {
    kind: "struct",
    fields: camelCaseIdlTypeDefStruct(rawIdlTypeDefTy.fields),
  };
}

function camelCaseIdlTypeDef(rawIdlTypeDef: RawIdlTypeDef): IdlTypeDef {
  return {
    name: rawIdlTypeDef.name,
    type: camelCaseIdlTypeDefTy(rawIdlTypeDef.type),
  };
}

function camelCaseState(rawIdlState?: RawIdlState): IdlState | undefined {
  if (rawIdlState === undefined) {
    return undefined;
  }
  return {
    struct: camelCaseIdlTypeDef(rawIdlState.struct),
    methods: camelCaseIdlInstructions(rawIdlState.methods),
  };
}

function camelCaseMaybeIdlTypeDefs(
  typeDefs?: RawIdlTypeDef[]
): IdlTypeDef[] | undefined {
  if (typeDefs === undefined) {
    return undefined;
  }
  return typeDefs.map(camelCaseIdlTypeDef);
}

function camelCaseIdlEventField(field: RawIdlEventField): IdlEventField {
  return {
    name: camelCase(field.name),
    type: camelCaseIdlType(field.type),
    index: field.index,
  };
}

function camelCaseIdlEventFields(fields: RawIdlEventField[]): IdlEventField[] {
  return fields.map(camelCaseIdlEventField);
}

function camelCaseEvent(event: RawIdlEvent): IdlEvent {
  return {
    name: event.name,
    fields: camelCaseIdlEventFields(event.fields),
  };
}

function camelCaseEvents(events?: RawIdlEvent[]): IdlEvent[] | undefined {
  if (events === undefined) {
    return undefined;
  }
  return events.map(camelCaseEvent);
}

function camelCaseConstant(constant: RawIdlConstant): IdlConstant {
  return {
    name: constant.name,
    type: camelCaseIdlType(constant.type),
    value: constant.value,
  };
}

function camelCaseConstants(
  constants?: RawIdlConstant[]
): IdlConstant[] | undefined {
  if (constants === undefined) {
    return undefined;
  }
  return constants.map(camelCaseConstant);
}

export function isCamelized(idl: Idl | RawIdl): idl is Idl {
  return (idl as Idl).camelized === true;
}

export function camelCaseIdl<IDL extends Idl = Idl>(rawIdl: RawIdl): IDL {
  return {
    version: rawIdl.version,
    name: camelCase(rawIdl.name),
    instructions: camelCaseIdlInstructions(rawIdl.instructions),
    state: camelCaseState(rawIdl.state),
    accounts: camelCaseMaybeIdlTypeDefs(rawIdl.accounts),
    types: camelCaseMaybeIdlTypeDefs(rawIdl.types),
    events: camelCaseEvents(rawIdl.events),
    errors: rawIdl.errors,
    constants: camelCaseConstants(rawIdl.constants),
    camelized: true,
  } as IDL;
}

// Deterministic IDL address as a function of the program id.
export async function idlAddress(programId: PublicKey): Promise<PublicKey> {
  const base = (await PublicKey.findProgramAddress([], programId))[0];
  return await PublicKey.createWithSeed(base, seed(), programId);
}

// Seed for generating the idlAddress.
export function seed(): string {
  return "anchor:idl";
}

// The on-chain account of the IDL.
export interface IdlProgramAccount {
  authority: PublicKey;
  data: Buffer;
}

const IDL_ACCOUNT_LAYOUT: borsh.Layout<IdlProgramAccount> = borsh.struct([
  borsh.publicKey("authority"),
  borsh.vecU8("data"),
]);

export function decodeIdlAccount(data: Buffer): IdlProgramAccount {
  return IDL_ACCOUNT_LAYOUT.decode(data);
}

export function encodeIdlAccount(acc: IdlProgramAccount): Buffer {
  const buffer = Buffer.alloc(1000); // TODO: use a tighter buffer.
  const len = IDL_ACCOUNT_LAYOUT.encode(acc, buffer);
  return buffer.slice(0, len);
}
