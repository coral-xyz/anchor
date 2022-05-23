const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { Keypair, SystemProgram, Transaction } = require("@solana/web3.js");

describe("basic-5", () => {
  const provider = anchor.AnchorProvider.local("http://localhost:8899", {
    // skipPreflight: true
  });

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Basic5;

  it("Is runs the constructor", async () => {
    // #region ctor
    // Initialize the program's state struct.
    await program.state.rpc.new({
      accounts: {
        authority: provider.wallet.publicKey,
      },
    });
    // #endregion ctor

    // Fetch the state struct from the network.
    // #region accessor
    const state = await program.state.fetch();
    // #endregion accessor

    assert.ok(state.count.eq(new anchor.BN(0)));
  });

  it("Executes a method on the program", async () => {
    // #region instruction
    await program.state.rpc.increment({
      accounts: {
        authority: provider.wallet.publicKey,
      },
    });
    // #endregion instruction
    const state = await program.state.fetch();
    assert.ok(state.count.eq(new anchor.BN(1)));
  });

  it("Executes a invalid method on the program that charges a bot tax 😈", async () => {
    // #region instruction
    const spamKeypair = Keypair.generate();
    const spamPubkey = spamKeypair.publicKey;
    await program.provider.connection.requestAirdrop(
      spamPubkey,
      1e6
    );
    let tx = program.state.instruction.incrementOutOfBounds({
      accounts: {
        botTax: provider.wallet.publicKey,
        payer: spamPubkey,
        systemProgram: SystemProgram.programId,
      },
      signers: [spamKeypair]
    });
    let txId = await program.provider.connection.sendTransaction(
      new Transaction().add(tx),
      [spamKeypair],
      {
        skipPreflight: true,
        commitment: "confirmed"
      }
    );
    console.log("Txid:", txId);
    await new Promise(r => setTimeout(r, 10*1000));
    console.log((await program.provider.connection.getConfirmedTransaction(
      txId
    )).meta.logMessages);
    // #endregion instruction
    const state = await program.state.fetch();
    assert.ok(state.count.eq(new anchor.BN(1)));
  });
});
