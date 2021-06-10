import camelCase from "camelcase";
import { Layout } from "buffer-layout";
import * as borsh from "@project-serum/borsh";
import * as bs58 from "bs58";
import { Idl, IdlField, IdlStateMethod, IdlType, IdlTypeDef } from "../idl";
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
  // Instruction args layout. Maps namespaced method
  private ixLayout: Map<string, Layout>;

  // Base58 encoded sighash to instruction layout.
  private sighashLayouts: Map<string, { layout: Layout; name: string }>;

  public constructor(private idl: Idl) {
    this.ixLayout = InstructionCoder.parseIxLayout(idl);

    const sighashLayouts = new Map();
    idl.instructions.forEach((ix) => {
      const sh = sighash(SIGHASH_GLOBAL_NAMESPACE, ix.name);
      sighashLayouts.set(bs58.encode(sh), {
        layout: this.ixLayout.get(ix.name),
        name: ix.name,
      });
    });

    if (idl.state) {
      idl.state.methods.map((ix) => {
        const sh = sighash(SIGHASH_STATE_NAMESPACE, ix.name);
        sighashLayouts.set(bs58.encode(sh), {
          layout: this.ixLayout.get(ix.name) as Layout,
          name: ix.name,
        });
      });
    }

    this.sighashLayouts = sighashLayouts;
  }

  public decode(ix: Buffer | string): Instruction | null {
    if (typeof ix === "string") {
      ix = bs58.decode(ix);
    }
    let sighash = bs58.encode(ix.slice(0, 8));
    let data = ix.slice(8);
    const decoder = this.sighashLayouts.get(sighash);
    if (!decoder) {
      return null;
    }
    return {
      data: decoder.layout.decode(data),
      name: decoder.name,
    };
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

  public formatFields(
    ix: Instruction
  ): { name: string; type: string; data: string }[] {
    const idlIx = this.idl.instructions.filter((i) => ix.name === i.name)[0];
    if (idlIx === undefined) {
      throw new Error("Invalid instruction given");
    }
    return idlIx.args.map((idlField) => {
      return {
        name: idlField.name,
        type: this.formatIdlType(idlField.type),
        data: this.formatIdlData(
          idlField,
          ix.data[idlField.name],
          this.idl.types
        ),
      };
    });
  }

  private formatIdlType(idlType: IdlType): string {
    if (typeof idlType === "string") {
      return idlType as string;
    }

    // @ts-ignore
    if (idlType.vec) {
      // @ts-ignore
      return `vec<${this.formatIdlType(idlType.vec)}>`;
    }
    // @ts-ignore
    if (idlType.option) {
      // @ts-ignore
      return `option<${this.formatIdlType(idlType.option)}>`;
    }
    // @ts-ignore
    if (idlType.defined) {
      // @ts-ignore
      return idlType.defined;
    }
  }

  private formatIdlData(
    idlField: IdlField,
    data: Object,
    types?: IdlTypeDef[]
  ): string {
    if (typeof idlField.type === "string") {
      return data.toString();
    }
    // @ts-ignore
    if (idlField.type.vec) {
      // @ts-ignore
      return (
        "[" +
        data
          .map((d) =>
            this.formatIdlData(
              // @ts-ignore
              { name: "", type: idlField.type.vec },
              d
            )
          )
          .join(", ") +
        "]"
      );
    }
    // @ts-ignore
    if (idlField.type.option) {
      // @ts-ignore
      return data === null
        ? "null"
        : this.formatIdlData(
            // @ts-ignore
            { name: "", type: idlField.type.option },
            data
          );
    }
    // @ts-ignore
    if (idlField.type.defined) {
      if (types === undefined) {
        throw new Error("User defined types not provided");
      }
      // @ts-ignore
      const filtered = types.filter((t) => t.name === idlField.type.defined);
      if (filtered.length !== 1) {
        // @ts-ignore
        throw new Error(`Type not found: ${idlField.type.defined}`);
      }
      return this.formatIdlDataDefined(filtered[0], data, types);
    }

    return "unknown";
  }

  private formatIdlDataDefined(
    typeDef: IdlTypeDef,
    data: Object,
    types: IdlTypeDef[]
  ): string {
    console.log(data);
    if (typeDef.type.kind === "struct") {
      const fields = Object.keys(data)
        .map((k) => {
          const f = typeDef.type.fields.filter((f) => f.name === k)[0];
          if (f === undefined) {
            throw new Error("Unable to find type");
          }
          return k + ": " + this.formatIdlData(f, data[k], types);
        })
        .join(", ");
      return "{ " + fields + " }";
    } else {
      // todo
      return "{}";
    }
  }
}

export type Instruction = {
  name: string;
  data: Object;
};
