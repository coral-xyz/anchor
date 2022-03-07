import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { AnchorCpiReturn } from "../target/types/anchor_cpi_return";

describe("anchor-cpi-return", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.AnchorCpiReturn as Program<AnchorCpiReturn>;

  it("Is initialized!", async () => {
    const tx = await program.rpc.initialize({});
    console.log(program);
    console.log("Your transaction signature", tx);
  });
});
