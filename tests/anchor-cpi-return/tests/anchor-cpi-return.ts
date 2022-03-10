import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { AnchorCpiReturn } from "../target/types/anchor_cpi_return";
import { AnchorCpiCaller } from "../target/types/anchor_cpi_caller";

const { SystemProgram } = anchor.web3;

describe("anchor-cpi-return", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const callerProgram = anchor.workspace
    .AnchorCpiCaller as Program<AnchorCpiCaller>;
  const returnProgram = anchor.workspace
    .AnchorCpiReturn as Program<AnchorCpiReturn>;

  it("Is initialized!", async () => {
    const cpiReturn = anchor.web3.Keypair.generate();
    await returnProgram.methods
      .initialize()
      .accounts({
        account: cpiReturn.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([cpiReturn])
      .rpc();
    const tx = await callerProgram.methods
      .cpiCallReturnU64()
      .accounts({
        cpiReturn: cpiReturn.publicKey,
        cpiReturnProgram: returnProgram.programId,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
