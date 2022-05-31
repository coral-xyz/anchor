import * as anchor from "@project-serum/anchor";
import { parseTokenAccount, token } from "@project-serum/common";
import { Token } from "@solana/spl-token";
import { Keypair, PublicKey, sendAndConfirmTransaction, Transaction } from "@solana/web3.js";
import { Program } from "@project-serum/anchor";
import { PdaPayer } from "../target/types/pda_payer";
import { expect } from "chai";

function sleep(time: number) {
  return new Promise((resolve) => setTimeout(resolve, time));
}

describe("typescript", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.PdaPayer as Program<PdaPayer>;

  it('initWithPayer', async () => {
    const mint = new PublicKey('So11111111111111111111111111111111111111112');
    const tokenOwner = Keypair.generate().publicKey;
    const myAccount = Keypair.generate();
    const myAnotherAccount = Keypair.generate();
    await program.methods
      .initWithPayer()
      .accounts({
        normalPayer: provider.wallet.publicKey,
        mint,
        tokenOwner,
        myAccount: myAccount.publicKey,
        myAnotherAccount: myAnotherAccount.publicKey,
      })
      .signers([myAccount, myAnotherAccount])
      .rpc();
    
    const _myAccount = parseTokenAccount((await provider.connection.getAccountInfo(myAccount.publicKey)).data);
    expect(_myAccount.mint.equals(mint)).is.true;
    expect(_myAccount.owner.equals(tokenOwner)).is.true;

    const _myAnotherAccount = parseTokenAccount((await provider.connection.getAccountInfo(myAccount.publicKey)).data);
    expect(_myAnotherAccount.mint.equals(mint)).is.true;
    expect(_myAnotherAccount.owner.equals(tokenOwner)).is.true;
  });

  it('initIfNeededWithPayer', async () => {
    const mint = new PublicKey('So11111111111111111111111111111111111111112');
    const tokenOwner = Keypair.generate().publicKey;
    const myAccount = Keypair.generate();
    const myAnotherAccount = Keypair.generate();
    const numAttemps = 2;
    for (let i = 0; i < numAttemps; i += 1) {
      await program.methods
        .initIfNeededWithPayer()
        .accounts({
          normalPayer: provider.wallet.publicKey,
          mint,
          tokenOwner,
          myAccount: myAccount.publicKey,
          myAnotherAccount: myAnotherAccount.publicKey,
        })
        .signers([myAccount, myAnotherAccount])
        .rpc();
      
      const _myAccount = parseTokenAccount((await provider.connection.getAccountInfo(myAccount.publicKey)).data);
      expect(_myAccount.mint.equals(mint)).is.true;
      expect(_myAccount.owner.equals(tokenOwner)).is.true;

      const _myAnotherAccount = parseTokenAccount((await provider.connection.getAccountInfo(myAccount.publicKey)).data);
      expect(_myAnotherAccount.mint.equals(mint)).is.true;
      expect(_myAnotherAccount.owner.equals(tokenOwner)).is.true;

      if (i < numAttemps - 1) await sleep(2000);
    }
  });
  
  it('initWithPdaAsPayer', async () => {
    const otherProgram = Keypair.generate().publicKey;
    const [normalPda] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeSeeds")],
      otherProgram,
    );
    const [pdaPayer] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeOtherSeeds")],
      program.programId,
    );
    const mint = new PublicKey('So11111111111111111111111111111111111111112');
    const tokenOwner = Keypair.generate().publicKey;
    const normalPayerAccount = Keypair.generate();
    const pdaPayerAccount = Keypair.generate();

    // fund the pda payer
    const rentExemptAmount = await Token.getMinBalanceRentForExemptAccount(provider.connection);
    const tx = await provider.connection.requestAirdrop(pdaPayer, rentExemptAmount);
    await provider.connection.confirmTransaction(tx);

    const pdaPayerBalanceBefore = await provider.connection.getBalance(pdaPayer);

    await program.methods
      .initWithPdaAsPayer()
      .accounts({
        normalPayer: provider.wallet.publicKey,
        pdaPayer,
        normalPda,
        mint,
        tokenOwner,
        normalPayerAccount: normalPayerAccount.publicKey,
        pdaPayerAccount: pdaPayerAccount.publicKey,
        otherProgram,
      })
      .signers([normalPayerAccount, pdaPayerAccount])
      .rpc();

    const pdaPayerBalanceAfter = await provider.connection.getBalance(pdaPayer);

    // pda payer must pay rent only
    expect(pdaPayerBalanceBefore - pdaPayerBalanceAfter == rentExemptAmount).is.true;

    const _normalPayerAccount = parseTokenAccount((await provider.connection.getAccountInfo(normalPayerAccount.publicKey)).data);
    expect(_normalPayerAccount.mint.equals(mint)).is.true;
    expect(_normalPayerAccount.owner.equals(tokenOwner)).is.true;

    const _pdaPayerAccount = parseTokenAccount((await provider.connection.getAccountInfo(pdaPayerAccount.publicKey)).data);
    expect(_pdaPayerAccount.mint.equals(mint)).is.true;
    expect(_pdaPayerAccount.owner.equals(tokenOwner)).is.true;
  });

  it('initIfNeededWithPdaAsPayer', async () => {
    const otherProgram = Keypair.generate().publicKey;
    const [normalPda] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeSeeds")],
      otherProgram,
    );
    const [pdaPayer] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeOtherSeeds")],
      program.programId,
    );
    const mint = new PublicKey('So11111111111111111111111111111111111111112');
    const tokenOwner = Keypair.generate().publicKey;
    const normalPayerAccount = Keypair.generate();
    const pdaPayerAccount = Keypair.generate();

    const numAttemps = 2;

    // fund the pda payer
    const rentExemptAmount = await Token.getMinBalanceRentForExemptAccount(provider.connection);
    const tx = await provider.connection.requestAirdrop(pdaPayer, numAttemps * rentExemptAmount);
    await provider.connection.confirmTransaction(tx);

    for (let i = 0; i < numAttemps; i += 1) {
      const pdaPayerBalanceBefore = await provider.connection.getBalance(pdaPayer);

      await program.methods
        .initIfNeededWithPdaAsPayer()
        .accounts({
          normalPayer: provider.wallet.publicKey,
          pdaPayer,
          normalPda,
          mint,
          tokenOwner,
          normalPayerAccount: normalPayerAccount.publicKey,
          pdaPayerAccount: pdaPayerAccount.publicKey,
          otherProgram,
        })
        .signers([normalPayerAccount, pdaPayerAccount])
        .rpc();

      const pdaPayerBalanceAfter = await provider.connection.getBalance(pdaPayer);

      if (i == 0) {
        // pda payer must pay rent only
        expect(pdaPayerBalanceBefore - pdaPayerBalanceAfter == rentExemptAmount).is.true;
      } else {
        // pda payer must be unchanged
        expect(pdaPayerBalanceAfter).equal(pdaPayerBalanceBefore);
      }

      const _normalPayerAccount = parseTokenAccount((await provider.connection.getAccountInfo(normalPayerAccount.publicKey)).data);
      expect(_normalPayerAccount.mint.equals(mint)).is.true;
      expect(_normalPayerAccount.owner.equals(tokenOwner)).is.true;

      const _pdaPayerAccount = parseTokenAccount((await provider.connection.getAccountInfo(pdaPayerAccount.publicKey)).data);
      expect(_pdaPayerAccount.mint.equals(mint)).is.true;
      expect(_pdaPayerAccount.owner.equals(tokenOwner)).is.true;

      if (i < numAttemps - 1) await sleep(2000);
    }
  });
})