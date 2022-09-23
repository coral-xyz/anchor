import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Idl } from "../../";
import {
  IdlField,
  IdlInstruction,
  IdlType,
  IdlTypeDef,
  IdlTypeDefTyEnum,
  IdlTypeDefTyStruct,
} from "../../idl";
import { Accounts, Context } from "../context";
import { MethodsBuilder } from "./methods";

/**
 * All instructions for an IDL.
 */
export type AllInstructions<IDL extends Idl> = IDL["instructions"][number];

/**
 * Returns a type of instruction name to the IdlInstruction.
 */
export type InstructionMap<I extends IdlInstruction> = {
  [K in I["name"]]: I & { name: K };
};

/**
 * Returns a type of instruction name to the IdlInstruction.
 */
export type AllInstructionsMap<IDL extends Idl> = InstructionMap<
  AllInstructions<IDL>
>;

/**
 * All accounts for an IDL.
 */
export type AllAccounts<IDL extends Idl> = IDL["accounts"] extends undefined
  ? IdlTypeDef
  : NonNullable<IDL["accounts"]>[number];

/**
 * Returns a type of instruction name to the IdlInstruction.
 */
export type AccountMap<I extends IdlTypeDef> = {
  [K in I["name"]]: I & { name: K };
};

/**
 * Returns a type of instruction name to the IdlInstruction.
 */
export type AllAccountsMap<IDL extends Idl> = AccountMap<AllAccounts<IDL>>;

export type MakeInstructionsNamespace<
  IDL extends Idl,
  I extends IdlInstruction,
  Ret,
  Mk extends { [M in keyof InstructionMap<I>]: unknown } = {
    [M in keyof InstructionMap<I>]: unknown;
  }
> = {
  [M in keyof InstructionMap<I>]: InstructionContextFn<
    IDL,
    InstructionMap<I>[M],
    Ret
  > &
    Mk[M];
};

export type MakeMethodsNamespace<IDL extends Idl, I extends IdlInstruction> = {
  [M in keyof InstructionMap<I>]: MethodsFn<
    IDL,
    InstructionMap<I>[M],
    MethodsBuilder<IDL, InstructionMap<I>[M]>
  >;
};

export type InstructionContextFn<
  IDL extends Idl,
  I extends AllInstructions<IDL>,
  Ret
> = (...args: InstructionContextFnArgs<IDL, I>) => Ret;

export type InstructionContextFnArgs<
  IDL extends Idl,
  I extends IDL["instructions"][number]
> = [
  ...ArgsTuple<I["args"], IdlTypes<IDL>>,
  Context<Accounts<I["accounts"][number]>>
];

export type InstructionAccountAddresses<
  IDL extends Idl,
  I extends AllInstructions<IDL>
> = {
  [N in keyof Accounts<I["accounts"][number]>]: PublicKey;
};

export type MethodsFn<
  IDL extends Idl,
  I extends IDL["instructions"][number],
  Ret
> = (...args: ArgsTuple<I["args"], IdlTypes<IDL>>) => Ret;

type TypeMap = {
  publicKey: PublicKey;
  bool: boolean;
  string: string;
  bytes: Buffer;
} & {
  [K in "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "f32" | "f64"]: number;
} & {
  [K in "u64" | "i64" | "u128" | "i128"]: BN;
};

export type DecodeType<T extends IdlType, Defined> = T extends keyof TypeMap
  ? TypeMap[T]
  : T extends { defined: keyof Defined }
  ? Defined[T["defined"]]
  : T extends { option: { defined: keyof Defined } }
  ? Defined[T["option"]["defined"]] | null
  : T extends { option: keyof TypeMap }
  ? TypeMap[T["option"]] | null
  : T extends { coption: { defined: keyof Defined } }
  ? Defined[T["coption"]["defined"]] | null
  : T extends { coption: keyof TypeMap }
  ? TypeMap[T["coption"]] | null
  : T extends { vec: keyof TypeMap }
  ? TypeMap[T["vec"]][]
  : T extends { vec: { defined: keyof Defined } }
  ? Defined[T["vec"]["defined"]][]
  : T extends { array: [defined: keyof TypeMap, size: number] }
  ? TypeMap[T["array"][0]][]
  : unknown;

/**
 * Tuple of arguments.
 */
type ArgsTuple<A extends IdlField[], Defined> = {
  [K in keyof A]: A[K] extends IdlField
    ? DecodeType<A[K]["type"], Defined>
    : unknown;
} & unknown[];

type FieldsOfType<I extends IdlTypeDef> = NonNullable<
  I["type"] extends IdlTypeDefTyStruct
    ? I["type"]["fields"]
    : I["type"] extends IdlTypeDefTyEnum
    ? I["type"]["variants"][number]["fields"]
    : any[]
>[number];

export type TypeDef<I extends IdlTypeDef, Defined> = {
  [F in FieldsOfType<I> as F["name"]]: DecodeType<F["type"], Defined>;
};

type TypeDefDictionary<T extends IdlTypeDef[], Defined> = {
  [K in T[number] as K["name"]]: TypeDef<K, Defined>;
};

type NestedTypeDefDictionary<T extends IdlTypeDef[]> = {
  [Outer in T[number] as Outer["name"]]: TypeDef<
    Outer,
    {
      [Inner in T[number] as Inner["name"]]: Inner extends Outer
        ? never
        : TypeDef<Inner, Record<string, never>>;
    }
  >;
};

export type IdlTypes<T extends Idl> = NestedTypeDefDictionary<
  NonNullable<T["types"]>
>;

export type IdlAccounts<T extends Idl> = TypeDefDictionary<
  NonNullable<T["accounts"]>,
  IdlTypes<T>
>;

export type IdlErrorInfo<IDL extends Idl> = NonNullable<IDL["errors"]>[number];
