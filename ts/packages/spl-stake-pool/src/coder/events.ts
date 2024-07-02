import { Idl, EventCoder, NoEventCoder } from "@coral-xyz/anchor";

export class SplStakePoolEventsCoder extends NoEventCoder implements EventCoder {
  constructor(_idl: Idl) {
    super();
  }
}
