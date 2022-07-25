import * as anchor from "@project-serum/anchor";
import { parseTokenAccount } from "@project-serum/common";
import { Token } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
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

  it("initWithPayer", async () => {
    const mint = new PublicKey("So11111111111111111111111111111111111111112");
    const tokenOwner = Keypair.generate().publicKey;
    const myAccount = Keypair.generate();
    const myAnotherAccount = Keypair.generate();
    const myProgramAccount = Keypair.generate();
    const [myPdaAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("PdaAccountSeeds")],
      program.programId
    );
    await program.methods
      .initWithPayer()
      .accounts({
        normalPayer: provider.wallet.publicKey,
        mint,
        tokenOwner,
        myAccount: myAccount.publicKey,
        myAnotherAccount: myAnotherAccount.publicKey,
        myProgramAccount: myProgramAccount.publicKey,
        myPdaAccount,
      })
      .signers([myAccount, myAnotherAccount, myProgramAccount])
      .rpc();

    const _myAccount = parseTokenAccount(
      (await provider.connection.getAccountInfo(myAccount.publicKey)).data
    );
    expect(_myAccount.mint.equals(mint)).is.true;
    expect(_myAccount.owner.equals(tokenOwner)).is.true;

    const _myAnotherAccount = parseTokenAccount(
      (await provider.connection.getAccountInfo(myAccount.publicKey)).data
    );
    expect(_myAnotherAccount.mint.equals(mint)).is.true;
    expect(_myAnotherAccount.owner.equals(tokenOwner)).is.true;

    const _myProgramAccount = await program.account.programAccount.fetch(
      myProgramAccount.publicKey
    );
    expect(_myProgramAccount.foo.eq(new anchor.BN(42))).is.true;

    const _myPdaAccount = await program.account.programAccount.fetch(
      myPdaAccount
    );
    expect(_myPdaAccount.foo.eq(new anchor.BN(42))).is.true;
  });

  it("initIfNeededWithPayer", async () => {
    const mint = new PublicKey("So11111111111111111111111111111111111111112");
    const tokenOwner = Keypair.generate().publicKey;
    const myAccount = Keypair.generate();
    const myAnotherAccount = Keypair.generate();
    const myProgramAccount = Keypair.generate();
    const [myPdaAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("PdaAccountSeeds")],
      program.programId
    );
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
          myProgramAccount: myProgramAccount.publicKey,
          myPdaAccount,
        })
        .signers([myAccount, myAnotherAccount, myProgramAccount])
        .rpc();

      const _myAccount = parseTokenAccount(
        (await provider.connection.getAccountInfo(myAccount.publicKey)).data
      );
      expect(_myAccount.mint.equals(mint)).is.true;
      expect(_myAccount.owner.equals(tokenOwner)).is.true;

      const _myAnotherAccount = parseTokenAccount(
        (await provider.connection.getAccountInfo(myAccount.publicKey)).data
      );
      expect(_myAnotherAccount.mint.equals(mint)).is.true;
      expect(_myAnotherAccount.owner.equals(tokenOwner)).is.true;

      const _myProgramAccount = await program.account.programAccount.fetch(
        myProgramAccount.publicKey
      );
      expect(_myProgramAccount.foo.eq(new anchor.BN(42))).is.true;

      const _myPdaAccount = await program.account.programAccount.fetch(
        myPdaAccount
      );
      expect(_myPdaAccount.foo.eq(new anchor.BN(42))).is.true;

      if (i < numAttemps - 1) await sleep(2000);
    }
  });

  it("initWithPdaAsPayer", async () => {
    const otherProgram = Keypair.generate().publicKey;
    const [normalPda] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeSeeds")],
      otherProgram
    );
    const [pdaPayer] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeOtherSeeds")],
      program.programId
    );
    const mint = new PublicKey("So11111111111111111111111111111111111111112");
    const tokenOwner = Keypair.generate().publicKey;
    const normalPayerAccount = Keypair.generate();
    const normalPayerProgramAccount = Keypair.generate();
    const [normalPayerPdaAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("NormalPayerPdaAccountSeeds")],
      program.programId
    );
    const pdaPayerAccount = Keypair.generate();
    const pdaPayerProgramAccount = Keypair.generate();
    const [pdaPayerPdaAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("PdaPayerPdaAccountSeeds")],
      program.programId
    );

    // fund the pda payer
    let rentExemptAmount = await Token.getMinBalanceRentForExemptAccount(
      provider.connection
    );
    rentExemptAmount +=
      2 * (await provider.connection.getMinimumBalanceForRentExemption(8 + 8));
    const tx = await provider.connection.requestAirdrop(
      pdaPayer,
      rentExemptAmount
    );
    await provider.connection.confirmTransaction(tx);

    const pdaPayerBalanceBefore = await provider.connection.getBalance(
      pdaPayer
    );

    await program.methods
      .initWithPdaAsPayer()
      .accounts({
        normalPayer: provider.wallet.publicKey,
        pdaPayer,
        normalPda,
        mint,
        tokenOwner,
        normalPayerAccount: normalPayerAccount.publicKey,
        normalPayerProgramAccount: normalPayerProgramAccount.publicKey,
        normalPayerPdaAccount,
        pdaPayerAccount: pdaPayerAccount.publicKey,
        pdaPayerProgramAccount: pdaPayerProgramAccount.publicKey,
        pdaPayerPdaAccount,
        otherProgram,
      })
      .signers([
        normalPayerAccount,
        normalPayerProgramAccount,
        pdaPayerAccount,
        pdaPayerProgramAccount,
      ])
      .rpc();

    const pdaPayerBalanceAfter = await provider.connection.getBalance(pdaPayer);

    // pda payer must pay rent only
    expect(pdaPayerBalanceBefore - pdaPayerBalanceAfter == rentExemptAmount).is
      .true;

    const _normalPayerAccount = parseTokenAccount(
      (await provider.connection.getAccountInfo(normalPayerAccount.publicKey))
        .data
    );
    expect(_normalPayerAccount.mint.equals(mint)).is.true;
    expect(_normalPayerAccount.owner.equals(tokenOwner)).is.true;

    const _normalPayerProgramAccount =
      await program.account.programAccount.fetch(
        normalPayerProgramAccount.publicKey
      );
    expect(_normalPayerProgramAccount.foo.eq(new anchor.BN(42))).is.true;

    const _normalPayerPdaAccount = await program.account.programAccount.fetch(
      normalPayerPdaAccount
    );
    expect(_normalPayerPdaAccount.foo.eq(new anchor.BN(42))).is.true;

    const _pdaPayerAccount = parseTokenAccount(
      (await provider.connection.getAccountInfo(pdaPayerAccount.publicKey)).data
    );
    expect(_pdaPayerAccount.mint.equals(mint)).is.true;
    expect(_pdaPayerAccount.owner.equals(tokenOwner)).is.true;

    const _pdaPayerProgramAccount = await program.account.programAccount.fetch(
      pdaPayerProgramAccount.publicKey
    );
    expect(_pdaPayerProgramAccount.foo.eq(new anchor.BN(42))).is.true;

    const _pdaPayerPdaAccount = await program.account.programAccount.fetch(
      pdaPayerPdaAccount
    );
    expect(_pdaPayerPdaAccount.foo.eq(new anchor.BN(42))).is.true;
  });

  it("initIfNeededWithPdaAsPayer", async () => {
    const otherProgram = Keypair.generate().publicKey;
    const [normalPda] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeSeeds")],
      otherProgram
    );
    const [pdaPayer] = await PublicKey.findProgramAddress(
      [Buffer.from("SomeOtherSeeds")],
      program.programId
    );
    const mint = new PublicKey("So11111111111111111111111111111111111111112");
    const tokenOwner = Keypair.generate().publicKey;
    const normalPayerAccount = Keypair.generate();
    const normalPayerProgramAccount = Keypair.generate();
    const pdaPayerAccount = Keypair.generate();
    const pdaPayerProgramAccount = Keypair.generate();

    const numAttemps = 2;

    // fund the pda payer
    let rentExemptAmount = await Token.getMinBalanceRentForExemptAccount(
      provider.connection
    );
    rentExemptAmount +=
      await provider.connection.getMinimumBalanceForRentExemption(8 + 8);
    const tx = await provider.connection.requestAirdrop(
      pdaPayer,
      numAttemps * rentExemptAmount
    );
    await provider.connection.confirmTransaction(tx);

    for (let i = 0; i < numAttemps; i += 1) {
      const pdaPayerBalanceBefore = await provider.connection.getBalance(
        pdaPayer
      );

      await program.methods
        .initIfNeededWithPdaAsPayer()
        .accounts({
          normalPayer: provider.wallet.publicKey,
          pdaPayer,
          normalPda,
          mint,
          tokenOwner,
          normalPayerAccount: normalPayerAccount.publicKey,
          normalPayerProgramAccount: normalPayerProgramAccount.publicKey,
          pdaPayerAccount: pdaPayerAccount.publicKey,
          pdaPayerProgramAccount: pdaPayerProgramAccount.publicKey,
          otherProgram,
        })
        .signers([
          normalPayerAccount,
          normalPayerProgramAccount,
          pdaPayerAccount,
          pdaPayerProgramAccount,
        ])
        .rpc();

      const pdaPayerBalanceAfter = await provider.connection.getBalance(
        pdaPayer
      );

      if (i == 0) {
        // pda payer must pay rent only
        expect(pdaPayerBalanceBefore - pdaPayerBalanceAfter == rentExemptAmount)
          .is.true;
      } else {
        // pda payer must be unchanged
        expect(pdaPayerBalanceAfter).equal(pdaPayerBalanceBefore);
      }

      const _normalPayerAccount = parseTokenAccount(
        (await provider.connection.getAccountInfo(normalPayerAccount.publicKey))
          .data
      );
      expect(_normalPayerAccount.mint.equals(mint)).is.true;
      expect(_normalPayerAccount.owner.equals(tokenOwner)).is.true;

      const _normalPayerProgramAccount =
        await program.account.programAccount.fetch(
          normalPayerProgramAccount.publicKey
        );
      expect(_normalPayerProgramAccount.foo.eq(new anchor.BN(42))).is.true;

      const _pdaPayerAccount = parseTokenAccount(
        (await provider.connection.getAccountInfo(pdaPayerAccount.publicKey))
          .data
      );
      expect(_pdaPayerAccount.mint.equals(mint)).is.true;
      expect(_pdaPayerAccount.owner.equals(tokenOwner)).is.true;

      const _pdaPayerProgramAccount =
        await program.account.programAccount.fetch(
          pdaPayerProgramAccount.publicKey
        );
      expect(_pdaPayerProgramAccount.foo.eq(new anchor.BN(42))).is.true;

      if (i < numAttemps - 1) await sleep(2000);
    }
  });
});
