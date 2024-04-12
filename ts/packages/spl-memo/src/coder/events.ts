import { Idl, EventCoder, NoEventCoder } from "@coral-xyz/anchor";

export class SplMemoEventsCoder extends NoEventCoder implements EventCoder {
  constructor(_idl: Idl) {
    super();
  }
}
