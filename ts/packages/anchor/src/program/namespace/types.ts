import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { Idl } from "../../";
import {
  IdlAccounts as IdlIdlAccounts,
  IdlAccountItem,
  IdlEnumFields,
  IdlEnumFieldsNamed,
  IdlEnumFieldsTuple,
  IdlField,
  IdlInstruction,
  IdlType,
  IdlTypeDef,
  IdlTypeDefTyAlias,
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
> = InstructionAccountsAddresses<I["accounts"][number]>;

type InstructionAccountsAddresses<A extends IdlAccountItem = IdlAccountItem> = {
  [N in A["name"]]: InstructionAccountsAddress<A & { name: N }>;
};

type InstructionAccountsAddress<A extends IdlAccountItem> =
  A extends IdlIdlAccounts
    ? InstructionAccountsAddresses<A["accounts"][number]>
    : PublicKey;

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
  [K in "u64" | "i64" | "u128" | "i128" | "u256" | "i256"]: BN;
};

export type DecodeType<T extends IdlType, Defined> = IdlType extends T
  ? unknown
  : T extends keyof TypeMap
  ? TypeMap[T]
  : T extends { defined: keyof Defined }
  ? Defined[T["defined"]]
  : T extends { option: IdlType }
  ? DecodeType<T["option"], Defined> | null
  : T extends { coption: IdlType }
  ? DecodeType<T["coption"], Defined> | null
  : T extends { vec: IdlType }
  ? DecodeType<T["vec"], Defined>[]
  : T extends { array: [defined: IdlType, size: number] }
  ? DecodeType<T["array"][0], Defined>[]
  : unknown;

/**
 * Tuple of arguments.
 */
type ArgsTuple<A extends IdlField[], Defined> = {
  [K in keyof A]: A[K] extends IdlField
    ? DecodeType<A[K]["type"], Defined>
    : unknown;
} & unknown[];
/**
 * flat {a: number, b: {c: string}} into number | string
 */
type UnboxToUnion<T> = T extends (infer U)[]
  ? UnboxToUnion<U>
  : T extends Record<string, never> // empty object, eg: named enum variant without fields
  ? "__empty_object__"
  : T extends Record<string, infer V> // object with props, eg: struct
  ? UnboxToUnion<V>
  : T;

type SnakeToCamelCase<S extends string> = S extends `${infer T}_${infer U}`
  ? `${T}${Capitalize<SnakeToCamelCase<U>>}`
  : S;

/**
 * decode single enum.field
 */
declare type DecodeEnumField<F, Defined> = F extends IdlType
  ? DecodeType<F, Defined>
  : never;

/**
 * decode enum variant: named or tuple
 */
declare type DecodeEnumFields<
  F extends IdlEnumFields,
  Defined
> = F extends IdlEnumFieldsNamed
  ? {
      [F2 in F[number] as SnakeToCamelCase<F2["name"]>]: DecodeEnumField<
        F2["type"],
        Defined
      >;
    }
  : F extends IdlEnumFieldsTuple
  ? {
      [F3 in keyof F as Exclude<F3, keyof unknown[]>]: DecodeEnumField<
        F[F3],
        Defined
      >;
    }
  : Record<string, never>;

type DecodeEnumVariants<I extends IdlTypeDefTyEnum, Defined> = {
  [V in I["variants"][number] as Uncapitalize<V["name"]>]: DecodeEnumFields<
    NonNullable<V["fields"]>,
    Defined
  >;
};

type ValueOf<T> = T[keyof T];
type XorEnumVariants<T extends Record<string, unknown>> = ValueOf<{
  [K1 in keyof T]: {
    [K2 in Exclude<keyof T, K1>]?: never;
  } & { [K2 in K1]: T[K2] };
}>;

type DecodeEnum<I extends IdlTypeDefTyEnum, Defined> = XorEnumVariants<
  DecodeEnumVariants<I, Defined>
>;

type DecodeStruct<I extends IdlTypeDefTyStruct, Defined> = {
  [F in I["fields"][number] as F["name"]]: DecodeType<F["type"], Defined>;
};

type DecodeAlias<I extends IdlTypeDefTyAlias, Defined> = DecodeType<
  I["value"],
  Defined
>;

export type TypeDef<
  I extends IdlTypeDef,
  Defined
> = I["type"] extends IdlTypeDefTyEnum
  ? DecodeEnum<I["type"], Defined>
  : I["type"] extends IdlTypeDefTyStruct
  ? DecodeStruct<I["type"], Defined>
  : I["type"] extends IdlTypeDefTyAlias
  ? DecodeAlias<I["type"], Defined>
  : never;

type TypeDefDictionary<T extends IdlTypeDef[], Defined> = {
  [K in T[number] as K["name"]]: TypeDef<K, Defined>;
};

type DecodedHelper<T extends IdlTypeDef[], Defined> = {
  [D in T[number] as D["name"]]: TypeDef<D, Defined>;
};

type UnknownType = "__unknown_defined_type__";
/**
 * empty "defined" object to produce UnknownType instead of never/unknown during idl types decoding
 *  */
type EmptyDefined = Record<UnknownType, never>;

type RecursiveDepth2<
  T extends IdlTypeDef[],
  Defined = EmptyDefined,
  Decoded = DecodedHelper<T, Defined>
> = UnknownType extends UnboxToUnion<Decoded>
  ? RecursiveDepth3<T, DecodedHelper<T, Defined>>
  : Decoded;

type RecursiveDepth3<
  T extends IdlTypeDef[],
  Defined = EmptyDefined,
  Decoded = DecodedHelper<T, Defined>
> = UnknownType extends UnboxToUnion<Decoded>
  ? RecursiveDepth4<T, DecodedHelper<T, Defined>>
  : Decoded;

type RecursiveDepth4<
  T extends IdlTypeDef[],
  Defined = EmptyDefined
> = DecodedHelper<T, Defined>;

/**
 * typescript can't handle truly recursive type (RecursiveTypes instead of RecursiveDepth2).
 * Hence we're doing "recursion" of depth=4 manually
 *  */
type RecursiveTypes<
  T extends IdlTypeDef[],
  Defined = EmptyDefined,
  Decoded = DecodedHelper<T, Defined>
> =
  // check if some of decoded types is Unknown (not decoded properly)
  UnknownType extends UnboxToUnion<Decoded>
    ? RecursiveDepth2<T, DecodedHelper<T, Defined>>
    : Decoded;

export type IdlTypes<T extends Idl> = RecursiveTypes<NonNullable<T["types"]>>;

type IdlEventType<
  I extends Idl,
  Event extends NonNullable<I["events"]>[number],
  Defined
> = {
  [F in Event["fields"][number] as F["name"]]: DecodeType<F["type"], Defined>;
};

export type IdlEvents<I extends Idl, Defined = IdlTypes<I>> = {
  [E in NonNullable<I["events"]>[number] as E["name"]]: IdlEventType<
    I,
    E,
    Defined
  >;
};

export type IdlAccounts<T extends Idl> = TypeDefDictionary<
  NonNullable<T["accounts"]>,
  IdlTypes<T>
>;

export type IdlErrorInfo<IDL extends Idl> = NonNullable<IDL["errors"]>[number];
