import { Idl, IdlField, IdlTypeDef, IdlType } from "../idl.js";
import { IdlError } from "../error.js";

export function accountSize(idl: Idl, idlAccount: IdlTypeDef) {
  if (idlAccount.type.kind === "enum") {
    const variantSizes = idlAccount.type.variants.map((variant) => {
      if (!variant.fields) {
        return 0;
      }

      return variant.fields
        .map((f: IdlField | IdlType) => {
          // Unnamed enum variant
          if (!(typeof f === "object" && "name" in f)) {
            return typeSize(idl, f);
          }

          // Named enum variant
          return typeSize(idl, f.type);
        })
        .reduce((acc, size) => acc + size, 0);
    });

    return Math.max(...variantSizes) + 1;
  }

  return idlAccount.type.fields
    .map((f) => typeSize(idl, f.type))
    .reduce((acc, size) => acc + size, 0);
}

// Returns the size of the type in bytes. For variable length types, just return
// 1. Users should override this value in such cases.
function typeSize(idl: Idl, ty: IdlType): number {
  switch (ty) {
    case "bool":
      return 1;
    case "u8":
      return 1;
    case "i8":
      return 1;
    case "i16":
      return 2;
    case "u16":
      return 2;
    case "u32":
      return 4;
    case "i32":
      return 4;
    case "f32":
      return 4;
    case "u64":
      return 8;
    case "i64":
      return 8;
    case "f64":
      return 8;
    case "u128":
      return 16;
    case "i128":
      return 16;
    case "u256":
      return 32;
    case "i256":
      return 32;
    case "bytes":
      return 1;
    case "string":
      return 1;
    case "publicKey":
      return 32;
    default:
      if ("vec" in ty) {
        return 1;
      }
      if ("option" in ty) {
        return 1 + typeSize(idl, ty.option);
      }
      if ("coption" in ty) {
        return 4 + typeSize(idl, ty.coption);
      }
      if ("defined" in ty) {
        const filtered = idl.types?.filter((t) => t.name === ty.defined) ?? [];
        if (filtered.length !== 1) {
          throw new IdlError(`Type not found: ${JSON.stringify(ty)}`);
        }
        let typeDef = filtered[0];

        return accountSize(idl, typeDef);
      }
      if ("array" in ty) {
        let arrayTy = ty.array[0];
        let arraySize = ty.array[1];
        return typeSize(idl, arrayTy) * arraySize;
      }
      throw new Error(`Invalid type ${JSON.stringify(ty)}`);
  }
}
