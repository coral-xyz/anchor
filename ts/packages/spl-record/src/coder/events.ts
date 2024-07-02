import { Idl, EventCoder, NoEventCoder } from "@coral-xyz/anchor";

export class SplRecordEventsCoder extends NoEventCoder implements EventCoder {
  constructor(_idl: Idl) {
    super();
  }
}
