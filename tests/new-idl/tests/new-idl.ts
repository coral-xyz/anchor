import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NewIdl } from "../target/types/new_idl";

describe("new-idl", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.NewIdl as Program<NewIdl>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
