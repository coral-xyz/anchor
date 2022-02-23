import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { IdlDescriptions } from "../target/types/idl_descriptions";

describe("idl-descriptions", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.IdlDescriptions as Program<IdlDescriptions>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
