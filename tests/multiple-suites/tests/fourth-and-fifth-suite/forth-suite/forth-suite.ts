import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MultipleSuites } from "../../../target/types/multiple_suites";

describe("multiple-suites", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.MultipleSuites as Program<MultipleSuites>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize(new anchor.BN(34823), {});
    console.log("Your transaction signature", tx);
  });
});
