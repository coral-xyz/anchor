import camelCase from "camelcase";
import { Layout } from "buffer-layout";
import * as borsh from "@project-serum/borsh";
import * as bs58 from "bs58";
import { Idl, IdlField, IdlStateMethod } from "../idl";
import { IdlCoder } from "./idl";
import { sighash } from "./common";

/**
 * Namespace for state method function signatures.
 */
export const SIGHASH_STATE_NAMESPACE = "state";
/**
 * Namespace for global instruction function signatures (i.e. functions
 * that aren't namespaced by the state or any of its trait implementations).
 */
export const SIGHASH_GLOBAL_NAMESPACE = "global";

/**
 * Encodes and decodes program instructions.
 */
export class InstructionCoder {
  /**
   * Instruction args layout. Maps namespaced method
   */
  private ixLayout: Map<string, Layout>;

  // Base58 encoded sighash to instruction layout.
  private sighashLayouts: Map<string, Layout>;

  public constructor(idl: Idl) {
    this.ixLayout = InstructionCoder.parseIxLayout(idl);

    const sighashLayouts = new Map<string, Layout>();
    idl.instructions.forEach((ix) => {
      const sh = sighash(SIGHASH_GLOBAL_NAMESPACE, ix.name);
      sighashLayouts.set(bs58.encode(sh), this.ixLayout.get(ix.name));
    });

    if (idl.state) {
      idl.state.methods.map((ix) => {
        const sh = sighash(SIGHASH_STATE_NAMESPACE, ix.name);
        sighashLayouts.set(bs58.encode(sh), this.ixLayout.get(ix.name));
      });
    }

    this.sighashLayouts = sighashLayouts;
  }

  public decode(ix: Buffer | string): Object | undefined {
    if (typeof ix === "string") {
      ix = bs58.decode(ix);
    }
    let sighash = bs58.encode(ix.slice(0, 8));
    let data = ix.slice(8);
    return this.sighashLayouts.get(sighash)?.decode(data);
  }

  /**
   * Encodes a program instruction.
   */
  public encode(ixName: string, ix: any) {
    return this._encode(SIGHASH_GLOBAL_NAMESPACE, ixName, ix);
  }

  /**
   * Encodes a program state instruction.
   */
  public encodeState(ixName: string, ix: any) {
    return this._encode(SIGHASH_STATE_NAMESPACE, ixName, ix);
  }

  private _encode(nameSpace: string, ixName: string, ix: any): Buffer {
    const buffer = Buffer.alloc(1000); // TODO: use a tighter buffer.
    const methodName = camelCase(ixName);
    const len = this.ixLayout.get(methodName).encode(ix, buffer);
    const data = buffer.slice(0, len);
    return Buffer.concat([sighash(nameSpace, ixName), data]);
  }

  private static parseIxLayout(idl: Idl): Map<string, Layout> {
    const stateMethods = idl.state ? idl.state.methods : [];

    const ixLayouts = stateMethods
      .map((m: IdlStateMethod) => {
        let fieldLayouts = m.args.map((arg: IdlField) => {
          return IdlCoder.fieldLayout(arg, idl.types);
        });
        const name = camelCase(m.name);
        return [name, borsh.struct(fieldLayouts, name)];
      })
      .concat(
        idl.instructions.map((ix) => {
          let fieldLayouts = ix.args.map((arg: IdlField) =>
            IdlCoder.fieldLayout(arg, idl.types)
          );
          const name = camelCase(ix.name);
          return [name, borsh.struct(fieldLayouts, name)];
        })
      );
    // @ts-ignore
    return new Map(ixLayouts);
  }
}
