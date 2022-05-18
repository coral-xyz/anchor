import * as BufferLayout from "buffer-layout";
import {
  INSTRUCTION_LAYOUT,
  INSTRUCTION_LAYOUT_V2,
} from "@project-serum/serum/lib/instructions";
import camelCase from "camelcase";
import { Idl } from "src/idl";
import { InstructionCoder } from "..";

export class SerumInstructionCoder implements InstructionCoder {
  constructor(_: Idl) {}

  encode(ixName: string, ix: any): Buffer {
    switch (camelCase(ixName)) {
      case "initializeMarket": {
        return encodeInitializeMarket(ix);
      }
      default: {
        throw new Error(`Invalid instruction: ${ixName}`);
      }
    }
  }

  encodeState(_ixName: string, _ix: any): Buffer {
    throw new Error("SPL token does not have state");
  }
}

function encodeInitializeMarket({
  baseLotSize,
  quoteLotSize,
  feeRateBps,
  vaultSignerNonce,
  quoteDustThreshold,
}: any): Buffer {
  return encodeData({
    initializeMarket: {
      baseLotSize,
      quoteLotSize,
      feeRateBps,
      vaultSignerNonce,
      quoteDustThreshold,
    },
  });
}

function publicKey(property: string): any {
  return BufferLayout.blob(32, property);
}

function encodeData(instruction: any): Buffer {
  let b = Buffer.alloc(instructionMaxSpan);
  let span = INSTRUCTION_LAYOUT.encode(instruction, b);
  return b.slice(0, span);
}

function encodeDataV2(instruction: any): Buffer {
  let b = Buffer.alloc(instructionMaxSpan);
  let span = INSTRUCTION_LAYOUT_V2.encode(instruction, b);
  return b.slice(0, span);
}

const instructionMaxSpan = Math.max(
  // @ts-ignore
  ...Object.values(INSTRUCTION_LAYOUT.registry).map((r) => r.span)
);
