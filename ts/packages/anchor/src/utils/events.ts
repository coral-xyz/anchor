import {
  CompiledInnerInstruction,
  VersionedTransactionResponse,
} from "@solana/web3.js";
import { BorshEventCoder } from "src/coder";
import { Idl } from "src/idl";
import { IdlEvents } from "src/program";

// https://github.com/coral-xyz/anchor/blob/v0.29.0/tests/events/tests/events.ts#L61-L62
export function parseCpiEvents(
  eventDecoder: BorshEventCoder,
  transactionResponse: VersionedTransactionResponse
): { name: string; data: any }[] {
  const events: { name: string; data: any }[] = [];
  const inner: CompiledInnerInstruction[] =
    transactionResponse?.meta?.innerInstructions ?? [];
  const idlProgramId = eventDecoder.idlAddress;
  for (let i = 0; i < inner.length; i++) {
    for (let j = 0; j < inner[i].instructions.length; j++) {
      const ix = inner[i].instructions[j];
      const programPubkey =
        transactionResponse?.transaction.message.staticAccountKeys[
          ix.programIdIndex
        ];
      if (programPubkey === undefined || !programPubkey.equals(idlProgramId)) {
        // we are at instructions that does not match the linked program
        continue;
      }
      const event = eventDecoder.decodeCpi(ix.data);
      if (event) {
        events.push(event);
      }
    }
  }
  return events;
}
