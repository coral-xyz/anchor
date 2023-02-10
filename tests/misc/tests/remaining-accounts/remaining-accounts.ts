import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { RemainingAccounts } from "../../target/types/remaining_accounts";
import { SystemProgram, LAMPORTS_PER_SOL, Keypair } from "@solana/web3.js";
import { expect } from "chai";

describe("remaining-accounts", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RemainingAccounts as Program<RemainingAccounts>;

  const payer = Keypair.generate();

  const amount = 1000000000; 


  before(async () => {
    let airdropSignature1 = await provider.connection.requestAirdrop(
      payer.publicKey,
      LAMPORTS_PER_SOL*5,
    );
    await  provider.connection.confirmTransaction(airdropSignature1);
  })

  it("Deposit", async () => {
    
    const [payerPDA, payerBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("bank"), payer.publicKey.toBuffer()], 
      program.programId
    )

    let Pda_before = await provider.connection.getBalance(payerPDA)
    expect(Pda_before).to.equal(0)

    const tx = await program.methods.deposit(
      payerBump,
      new anchor.BN(amount),
    )
    .accounts({
        bank: payerPDA,
        payer: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId, 
    })
    .signers([payer])
    .rpc()

    let Pda_after = await provider.connection.getBalance(payerPDA)
    expect(Pda_after).to.greaterThanOrEqual(amount)
    
  });

  it("Withdraw", async () => {  
    const recepient1 = Keypair.generate();
    const recepient2 = Keypair.generate();
    
    const [payerPDA, payerBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("bank"), payer.publicKey.toBuffer()], 
      program.programId
    )

    const tx = await program.methods.withdraw()
    .accounts({
      bank: payerPDA,
      payer: payer.publicKey,
      systemProgram:  anchor.web3.SystemProgram.programId,
  }).remainingAccounts([
    { pubkey: recepient1.publicKey, isWritable: true, isSigner: false},
    { pubkey: recepient2.publicKey, isWritable: true, isSigner: false}
  ])
  .signers([payer])
  .rpc()

    let balRecepient1 = await provider.connection.getBalance(recepient1.publicKey)
    expect(balRecepient1).to.equal(amount/2)

    let balRecepient2 = await provider.connection.getBalance(recepient2.publicKey)
    expect(balRecepient2).to.equal(amount/2)
   
  });
 
});
