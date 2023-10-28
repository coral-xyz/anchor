import * as anchor from "@coral-xyz/anchor";
import { Idl } from "@coral-xyz/anchor";
import { expect } from "chai";

import { Generics, IDL } from "../target/types/generics";

describe("Generics", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  /**
   * Checks if the IDL produced by Anchor CLI is compatible with the TypeScript
   * `Idl` type. Detects a potential mismatch between Rust and TypeScript IDL
   * definitions.
   */
  it("Raw IDL", () => {
    const idl: Idl = IDL;
    expect(idl).to.not.be.undefined;
  });
});
