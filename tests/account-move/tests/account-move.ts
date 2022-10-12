import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { AccountMove } from "../target/types/account_move";

describe("account-move", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.AccountMove as Program<AccountMove>;

  it("Is initialized!", async () => {
    // Add your test here.
    // let testAccount = findProgramAddressSync(
    //   [Buffer.from("test_account")],
    //   program.programId
    // );
    // const tx = program.methods.initialize().accounts({});
    // console.log(tx);
    // await tx.rpc();
    const tx = await program.methods.initialize().accounts({}).rpc();
    console.log("Your transaction signature", tx);
  });
});
