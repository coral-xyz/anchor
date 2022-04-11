import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Idl } from "../../";
import {
  IdlEnumVariant,
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

type StructDef<I extends IdlField[], Defined> = {
  [F in I[number]["name"]]: DecodeType<
    (I[number] & { name: F })["type"],
    Defined
  >;
};

type VariantDef<
  T extends IdlEnumVariant,
  Defined
> = T["fields"] extends IdlField[]
  ? StructDef<T["fields"], Defined>
  : T["fields"] extends IdlType[]
  ? unknown // TODO: Tuple enum variants are not supported yet.
  : Record<string, never>;

type XorVariantDefs<T extends Record<string, unknown>> = ValueOf<{
  [K1 in keyof T]: {
    [K2 in Exclude<keyof T, K1>]?: never;
  } & { [K2 in K1]: T[K2] };
}>;
type VariantDefs<I extends IdlEnumVariant[], Defined> = {
  [K in I[number]["name"]]: VariantDef<I[number] & { name: K }, Defined>;
};
type EnumDef<I extends IdlEnumVariant[], Defined> = XorVariantDefs<
  VariantDefs<I, Defined>
>;

export type TypeDef<
  I extends IdlTypeDef,
  Defined
> = I["type"] extends IdlTypeDefTyStruct
  ? StructDef<I["type"]["fields"], Defined>
  : I["type"] extends IdlTypeDefTyEnum
  ? EnumDef<I["type"]["variants"], Defined>
  : never;

type ValueOf<T> = T[keyof T];
type FindTypeDeps<T extends IdlType> = T extends {
  defined: string;
}
  ? T["defined"]
  : T extends { option: { defined: string } }
  ? T["option"]["defined"]
  : T extends { coption: { defined: string } }
  ? T["coption"]["defined"]
  : never;
type FindStructDeps<I extends IdlTypeDefTyStruct> = FindTypeDeps<
  I["fields"][number]["type"]
>;
type FindEnumDeps<I extends IdlTypeDefTyEnum> = NonNullable<
  I["variants"][number]["fields"]
> extends infer T
  ? T extends IdlField[]
    ? FindTypeDeps<T[number]["type"]>
    : T extends IdlType[]
    ? FindTypeDeps<T[number]>
    : never
  : never;
type FindUserDefinedDeps<I extends IdlTypeDef> =
  I["type"] extends IdlTypeDefTyStruct
    ? FindStructDeps<I["type"]>
    : I["type"] extends IdlTypeDefTyEnum
    ? FindEnumDeps<I["type"]>
    : never;
type FindUserDefined<T extends Record<string, IdlTypeDef>> = ValueOf<{
  [K in keyof T]: FindUserDefinedDeps<T[K]>;
}>;
type UserDefinedDeps<T extends Record<string, IdlTypeDef>> = {
  [K in FindUserDefined<T>]: FindUserDefinedDeps<T[K]>;
};

type UserDefinedDicionary<T extends Record<string, IdlTypeDef>, Defined> = {
  [K in keyof Defined]: K extends keyof T
    ? Defined[K] extends keyof T
      ? TypeDef<
          T[K],
          UserDefinedDicionary<
            T,
            Pick<T, Defined[K]> & UserDefinedDeps<Pick<T, Defined[K]>>
          >
        >
      : TypeDef<T[K], Record<string, never>>
    : never;
};
type MapIdlTypeDefs<T extends IdlTypeDef[]> = {
  [K in T[number]["name"]]: T[number] & { name: K };
};

type TypeDefDictionary<T extends IdlTypeDef[], Defined> = {
  [K in T[number]["name"]]: TypeDef<T[number] & { name: K }, Defined>;
};

export type IdlTypes<T extends Idl> = TypeDefDictionary<
  NonNullable<T["types"]>,
  UserDefinedDicionary<
    MapIdlTypeDefs<NonNullable<T["types"]>>,
    UserDefinedDeps<MapIdlTypeDefs<NonNullable<T["types"]>>>
  >
>;

export type IdlAccounts<T extends Idl> = TypeDefDictionary<
  NonNullable<T["accounts"]>,
  UserDefinedDicionary<
    MapIdlTypeDefs<NonNullable<T["types"]>>,
    UserDefinedDeps<MapIdlTypeDefs<NonNullable<T["types"]>>>
  >
>;

export type IdlErrorInfo<IDL extends Idl> = NonNullable<IDL["errors"]>[number];
