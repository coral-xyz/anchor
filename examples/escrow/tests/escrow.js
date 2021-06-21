const anchor = require("@project-serum/anchor");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");
const assert = require("assert");

const {
  getTokenAccount,
  createMint,
  createTokenAccount,
  mintTo,
} = require("./utils");

describe("escrow", () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow;

  let mintA = null;
  let mintB = null;
  let initializerTokenAccountA = null;
  let initializerTokenAccountB = null;
  let takerTokenAccountA = null;
  let takerTokenAccountB = null;
  let pda = null;

  const takerAmount = 1000;
  const initializerAmount = 500;

  let escrowAccount = anchor.web3.Keypair.generate();

  it("Initialise test state", async () => {
    // Create the token A and B mints
    mintA = await createMint(provider);
    mintB = await createMint(provider);

    // Create token accounts for A
    initializerTokenAccountA = await createTokenAccount(
      provider,
      mintA,
      provider.wallet.publicKey
    );

    takerTokenAccountA = await createTokenAccount(
      provider,
      mintA,
      provider.wallet.publicKey
    );

    // Create token accounts for B
    initializerTokenAccountB = await createTokenAccount(
      provider,
      mintB,
      provider.wallet.publicKey
    );

    takerTokenAccountB = await createTokenAccount(
      provider,
      mintB,
      provider.wallet.publicKey
    );

    // Mint 500 of token A to initializer token account A
    await mintTo(
      provider,
      mintA,
      initializerTokenAccountA,
      new anchor.BN(initializerAmount),
      provider.wallet.publicKey
    );

    // Mint 1000 of token B to taker token account B
    await mintTo(
      provider,
      mintB,
      takerTokenAccountB,
      new anchor.BN(takerAmount),
      provider.wallet.publicKey
    );

    let _initializerTokenAccountA = await getTokenAccount(
      provider,
      initializerTokenAccountA
    );

    let _takerTokenAccountB = await getTokenAccount(
      provider,
      takerTokenAccountB
    );

    assert.ok(_initializerTokenAccountA.amount.toNumber() == initializerAmount);
    assert.ok(_takerTokenAccountB.amount.toNumber() == takerAmount);
  });

  it("Initialize escrow", async () => {
    await program.rpc.initializeEscrow(
      new anchor.BN(initializerAmount),
      new anchor.BN(takerAmount),
      {
        accounts: {
          initializer: provider.wallet.publicKey,
          initializerDepositTokenAccount: initializerTokenAccountA,
          initializerReceiveTokenAccount: initializerTokenAccountB,
          escrowAccount: escrowAccount.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(escrowAccount),
        ],
        signers: [escrowAccount],
      }
    );

    // Get the PDA that is assigned authority to token account
    const [_pda, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("escrow"))],
      program.programId
    );

    pda = _pda;

    let _initializerTokenAccountA = await getTokenAccount(
      provider,
      initializerTokenAccountA
    );

    let _escrowAccount = await program.account.escrowAccount.fetch(
      escrowAccount.publicKey
    );

    // Check that the new owner is the PDA
    assert.ok(_initializerTokenAccountA.owner.equals(pda));

    // Check that the values in the escrow account match what we expect
    assert.ok(_escrowAccount.initializerKey.equals(provider.wallet.publicKey));
    assert.ok(_escrowAccount.initializerAmount.toNumber() == initializerAmount);
    assert.ok(_escrowAccount.takerAmount.toNumber() == takerAmount);
    assert.ok(
      _escrowAccount.initializerDepositTokenAccount.equals(
        initializerTokenAccountA
      )
    );
    assert.ok(
      _escrowAccount.initializerReceiveTokenAccount.equals(
        initializerTokenAccountB
      )
    );
  });

  it("Exchange escrow", async () => {
    await program.rpc.exchange({
      accounts: {
        taker: provider.wallet.publicKey,
        takerDepositTokenAccount: takerTokenAccountB,
        takerReceiveTokenAccount: takerTokenAccountA,
        pdaDepositTokenAccount: initializerTokenAccountA,
        initializerReceiveTokenAccount: initializerTokenAccountB,
        initializerMainAccount: provider.wallet.publicKey,
        escrowAccount: escrowAccount.publicKey,
        pdaAccount: pda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    let _takerTokenAccountA = await getTokenAccount(
      provider,
      takerTokenAccountA
    );

    let _takerTokenAccountB = await getTokenAccount(
      provider,
      takerTokenAccountB
    );

    let _initializerTokenAccountA = await getTokenAccount(
      provider,
      initializerTokenAccountA
    );

    let _initializerTokenAccountB = await getTokenAccount(
      provider,
      initializerTokenAccountB
    );

    // Check that the initializer gets back ownership of their token account
    assert.ok(_takerTokenAccountA.owner.equals(provider.wallet.publicKey));

    assert.ok(_takerTokenAccountA.amount.toNumber() == initializerAmount);
    assert.ok(_initializerTokenAccountA.amount.toNumber() == 0);

    assert.ok(_initializerTokenAccountB.amount.toNumber() == takerAmount);
    assert.ok(_takerTokenAccountB.amount.toNumber() == 0);
  });

  let newEscrow = anchor.web3.Keypair.generate();

  it("Initialize escrow and cancel escrow", async () => {
    // Put back tokens into initializerTokenA
    await mintTo(
      provider,
      mintA,
      initializerTokenAccountA,
      new anchor.BN(initializerAmount),
      provider.wallet.publicKey
    );

    await program.rpc.initializeEscrow(
      new anchor.BN(initializerAmount),
      new anchor.BN(takerAmount),
      {
        accounts: {
          initializer: provider.wallet.publicKey,
          initializerDepositTokenAccount: initializerTokenAccountA,
          initializerReceiveTokenAccount: initializerTokenAccountB,
          escrowAccount: newEscrow.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        instructions: [
          await program.account.escrowAccount.createInstruction(newEscrow),
        ],
        signers: [newEscrow],
      }
    );

    let _initializerTokenAccountA = await getTokenAccount(
      provider,
      initializerTokenAccountA
    );

    // Check that the new owner is the PDA
    assert.ok(_initializerTokenAccountA.owner.equals(pda));

    // Cancel the escrow, and the owner should be the initializer
    // Get the PDA that is assigned authority to token account
    await program.rpc.cancelEscrow({
      accounts: {
        initializer: provider.wallet.publicKey,
        pdaDepositTokenAccount: initializerTokenAccountA,
        pdaAccount: pda,
        escrowAccount: newEscrow.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    _initializerTokenAccountA = await getTokenAccount(
      provider,
      initializerTokenAccountA
    );

    // Check that the new owner is the PDA
    assert.ok(
      _initializerTokenAccountA.owner.equals(provider.wallet.publicKey)
    );
  });
});
