import { Buffer } from "buffer";
import * as BufferLayout from "@solana/buffer-layout";

import * as Layout from "./layout";

/**
 * @internal
 */
export type InstructionType = {
  /** The Instruction index (from solana upstream program) */
  index: number;
  /** The BufferLayout to use to build data */
  layout: BufferLayout.Layout;
};

/**
 * Populate a buffer of instruction data using an InstructionType
 * @internal
 */
export function encodeData(type: InstructionType, fields?: any): Buffer {
  const allocLength =
    type.layout.span >= 0 ? type.layout.span : Layout.getAlloc(type, fields);
  const data = Buffer.alloc(allocLength);
  const layoutFields = Object.assign({ instruction: type.index }, fields);
  type.layout.encode(layoutFields, data);
  return data;
}
