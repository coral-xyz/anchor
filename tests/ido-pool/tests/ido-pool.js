const anchor = require("@project-serum/anchor");
const assert = require("assert");
const {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  Token,
} = require("@solana/spl-token");
const {
  sleep,
  getTokenAccount,
  createMint,
  createTokenAccount,
} = require("./utils");
const { token } = require("@project-serum/anchor/dist/cjs/utils");

describe("ido-pool", () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.IdoPool;

  // All mints default to 6 decimal places.
  const watermelonIdoAmount = new anchor.BN(5000000);

  // These are all of the variables we assume exist in the world already and
  // are available to the client.
  let usdcMintToken = null;
  let usdcMint = null;
  let watermelonMintToken = null;
  let watermelonMint = null;
  let idoAuthorityUsdc = null;
  let idoAuthorityWatermelon = null;

  it("Initializes the state-of-the-world", async () => {
    usdcMintToken = await createMint(provider);
    watermelonMintToken = await createMint(provider);
    usdcMint = usdcMintToken.publicKey;
    watermelonMint = watermelonMintToken.publicKey;
    idoAuthorityUsdc = await createTokenAccount(
      provider,
      usdcMint,
      provider.wallet.publicKey
    );
    idoAuthorityWatermelon = await createTokenAccount(
      provider,
      watermelonMint,
      provider.wallet.publicKey
    );
    // Mint Watermelon tokens the will be distributed from the IDO pool.
    await watermelonMintToken.mintTo(
      idoAuthorityWatermelon,
      provider.wallet.publicKey,
      [],
      watermelonIdoAmount.toString(),
    );
    idoAuthority_watermelon_account = await getTokenAccount(
      provider,
      idoAuthorityWatermelon
    );
    assert.ok(idoAuthority_watermelon_account.amount.eq(watermelonIdoAmount));
  });

  // These are all variables the client will have to create to initialize the
  // IDO pool
  let poolSigner = null;
  let redeemableMintToken = null;
  let redeemableMint = null;
  let poolWatermelon = null;
  let poolUsdc = null;
  // let idoAccount = null;

  let startIdoTs = null;
  let endDepositsTs = null;
  let endIdoTs = null;
  let idoName = "test_ido";

  it("Initializes the IDO pool", async () => {
    let bumps = new PoolBumps();

    const [idoAccount, idoAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName)],
      program.programId
    );
    bumps.idoAccount = idoAccountBump;

    const [redeemableMint, redeemableMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("redeemable_mint")],
      program.programId
    );
    bumps.redeemableMint = redeemableMintBump;

    const [poolWatermelon, poolWatermelonBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("pool_watermelon")],
      program.programId
    );
    bumps.poolWatermelon = poolWatermelonBump;

    const [poolUsdc, poolUsdcBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("pool_usdc")],
      program.programId
    );
    bumps.poolUsdc = poolUsdcBump;

    // Pool doesn't need a Redeemable SPL token account because it only
    // burns and mints redeemable tokens, it never stores them.
    // redeemableMintToken = await createMint(provider, poolSigner);
    // redeemableMint = redeemableMintToken.publicKey;
    // poolWatermelon = await createTokenAccount(
    //   provider,
    //   watermelonMint,
    //   poolSigner
    // );
    // poolUsdc = await createTokenAccount(provider, usdcMint, poolSigner);
    // console.log("bumps:", bumps);
    // console.log("program account", idoAccount.toString());
    // console.log("program id", program.programId.toString());

    // idoAccount = anchor.web3.Keypair.generate();
    const nowBn = new anchor.BN(Date.now() / 1000);
    startIdoTs = nowBn.add(new anchor.BN(5));
    endDepositsTs = nowBn.add(new anchor.BN(10));
    endIdoTs = nowBn.add(new anchor.BN(15));

    // await program.rpc.testPdas(
    //   bumps, {
    //   accounts: {
    //     idoAuthority: provider.wallet.publicKey,
    //     idoAuthorityWatermelon,
    //     idoAccount,
    //     systemProgram: anchor.web3.SystemProgram.programId,
    //   }
    // }
    // );

    // Atomically create the new account and initialize it with the program.
    await program.rpc.initializePool(
      idoName,
      bumps,
      watermelonIdoAmount,
      startIdoTs,
      endDepositsTs,
      endIdoTs,
      {
        accounts: {
          idoAuthority: provider.wallet.publicKey,
          idoAuthorityWatermelon,
          idoAccount,
          watermelonMint,
          usdcMint,
          redeemableMint,
          poolWatermelon,
          poolUsdc,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      }
    );

    idoAuthoritys_watermelon_account = await getTokenAccount(
      provider,
      idoAuthorityWatermelon
    );
    assert.ok(idoAuthoritys_watermelon_account.amount.eq(new anchor.BN(0)));
  });

  // We're going to need to start using the associated program account for creating token accounts
  // if not in testing, then definitely in production.

  let userUsdc = null;
  let userRedeemable = null;
  // 10 usdc
  const firstDeposit = new anchor.BN(10_000_349);

  it("Exchanges user USDC for redeemable tokens", async () => {
    // Wait until the IDO has opened.
    if (Date.now() < startIdoTs.toNumber() * 1000) {
      await sleep(startIdoTs.toNumber() * 1000 - Date.now() + 2000);
    }

    const [idoAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName)],
      program.programId
    );

    const [redeemableMint] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("redeemable_mint")],
      program.programId
    );

    const [poolUsdc] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("pool_usdc")],
      program.programId
    );


    userUsdc = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcMint,
      program.provider.wallet.publicKey
    );
    // Get the instructions to add to the RPC call
    let createUserUsdcInstr = Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcMint,
      userUsdc,
      program.provider.wallet.publicKey,
      program.provider.wallet.publicKey,
    )
    let createUserUsdcTrns = new anchor.web3.Transaction().add(createUserUsdcInstr);
    await provider.send(createUserUsdcTrns);
    await usdcMintToken.mintTo(
      userUsdc,
      provider.wallet.publicKey,
      [],
      firstDeposit.toString(),
    );

    // TODO just temporarily checking if we inited correctly
    userUsdcAccount = await getTokenAccount(provider, userUsdc);
    assert.ok(userUsdcAccount.amount.eq(firstDeposit));

    const [userRedeemable] = await anchor.web3.PublicKey.findProgramAddress(
      [provider.wallet.publicKey.toBuffer(),
      Buffer.from(idoName),
      Buffer.from("user_redeemable")],
      program.programId
    );

    // userRedeemable = await createTokenAccount(
    //   provider,
    //   redeemableMint,
    //   provider.wallet.publicKey
    // );

    try {
      const tx = await program.rpc.exchangeUsdcForRedeemable(firstDeposit, {
        accounts: {
          userAuthority: provider.wallet.publicKey,
          userUsdc,
          userRedeemable,
          idoAccount,
          usdcMint,
          redeemableMint,
          watermelonMint,
          poolUsdc,
          tokenProgram: TOKEN_PROGRAM_ID,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
        instructions: [
          program.instruction.initUserRedeemable({
            accounts: {
              userAuthority: provider.wallet.publicKey,
              userRedeemable,
              idoAccount,
              redeemableMint,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
              clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            }
          })
        ]
      });
    } catch (err) {
      console.log("This is the error message", err.toString());
    }
    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(firstDeposit));
    userRedeemableAccount = await getTokenAccount(provider, userRedeemable);
    assert.ok(userRedeemableAccount.amount.eq(firstDeposit));
  });

  // 23 usdc
  const secondDeposit = new anchor.BN(23_000_672);
  let totalPoolUsdc, secondUserKeypair, secondUserUsdc, secondUserRedeemable;

  it("Exchanges a second users USDC for redeemable tokens", async () => {

    const [idoAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName)],
      program.programId
    );

    const [redeemableMint] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("redeemable_mint")],
      program.programId
    );

    const [poolUsdc] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(idoName), Buffer.from("pool_usdc")],
      program.programId
    );

    secondUserKeypair = anchor.web3.Keypair.generate();

    transferSolInstr = anchor.web3.SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      lamports: 100_000_000_000,
      toPubkey: secondUserKeypair.publicKey
    });
    secondUserUsdc = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcMint,
      secondUserKeypair.publicKey
    )
    createSecondUserUsdcInstr = Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcMint,
      secondUserUsdc,
      secondUserKeypair.publicKey,
      provider.wallet.publicKey
    );
    let createSecondUserUsdcTrns = new anchor.web3.Transaction();
    createSecondUserUsdcTrns.add(transferSolInstr);
    createSecondUserUsdcTrns.add(createSecondUserUsdcInstr);
    await provider.send(createSecondUserUsdcTrns);
    await usdcMintToken.mintTo(
      secondUserUsdc,
      provider.wallet.publicKey,
      [],
      secondDeposit.toString(),
    )

    // TODO: delete, temporarily checking transfer went through ok
    secondUserUsdcAccount = await getTokenAccount(provider, secondUserUsdc);
    assert.ok(secondUserUsdcAccount.amount.eq(secondDeposit));

    // secondUserRedeemable = await createTokenAccount(
    //   provider,
    //   redeemableMint,
    //   provider.wallet.publicKey
    // );
    const [secondUserRedeemable] = await anchor.web3.PublicKey.findProgramAddress(
      [secondUserKeypair.publicKey.toBuffer(),
      Buffer.from(idoName),
      Buffer.from("user_redeemable")],
      program.programId
    );

    await program.rpc.exchangeUsdcForRedeemable(secondDeposit, {
      accounts: {
        userAuthority: secondUserKeypair.publicKey,
        userUsdc: secondUserUsdc,
        userRedeemable: secondUserRedeemable,
        idoAccount,
        usdcMint,
        redeemableMint,
        watermelonMint,
        poolUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
      instructions: [
        program.instruction.initUserRedeemable({
          accounts: {
            userAuthority: secondUserKeypair.publicKey,
            userRedeemable: secondUserRedeemable,
            idoAccount,
            redeemableMint,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          },
          // signers: [secondUserKeypair]
        })
      ],
      signers: [secondUserKeypair]
    });

    secondUserRedeemableAccount = await getTokenAccount(
      provider,
      secondUserRedeemable
    );
    assert.ok(secondUserRedeemableAccount.amount.eq(secondDeposit));

    totalPoolUsdc = firstDeposit.add(secondDeposit);
    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(totalPoolUsdc));

  });

  const firstWithdrawal = new anchor.BN(2_000_000);

  it("Exchanges user Redeemable tokens for USDC", async () => {
    await program.rpc.exchangeRedeemableForUsdc(firstWithdrawal, {
      accounts: {
        userAuthority: provider.wallet.publicKey,
        userUsdc,
        userRedeemable,
        idoAccount,
        usdcMint,
        redeemableMint,
        watermelonMint,
        poolUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
    });

    totalPoolUsdc = totalPoolUsdc.sub(firstWithdrawal);
    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(totalPoolUsdc));
    userUsdcAccount = await getTokenAccount(provider, userUsdc);
    assert.ok(userUsdcAccount.amount.eq(firstWithdrawal));
  });

  it("Exchanges user Redeemable tokens for watermelon", async () => {
    // Wait until the IDO has opened.
    if (Date.now() < endIdoTs.toNumber() * 1000) {
      await sleep(endIdoTs.toNumber() * 1000 - Date.now() + 3000);
    }
    let firstUserRedeemable = firstDeposit.sub(firstWithdrawal);
    userWatermelon = await createTokenAccount(
      provider,
      watermelonMint,
      provider.wallet.publicKey
    );

    await program.rpc.exchangeRedeemableForWatermelon(firstUserRedeemable, {
      accounts: {
        idoAccount: idoAccount.publicKey,
        poolSigner,
        redeemableMint,
        poolWatermelon,
        userAuthority: provider.wallet.publicKey,
        userWatermelon,
        userRedeemable,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
    });

    poolWatermelonAccount = await getTokenAccount(provider, poolWatermelon);
    let redeemedWatermelon = firstUserRedeemable
      .mul(watermelonIdoAmount)
      .div(totalPoolUsdc);
    let remainingWatermelon = watermelonIdoAmount.sub(redeemedWatermelon);
    assert.ok(poolWatermelonAccount.amount.eq(remainingWatermelon));
    userWatermelonAccount = await getTokenAccount(provider, userWatermelon);
    assert.ok(userWatermelonAccount.amount.eq(redeemedWatermelon));
  });

  it("Exchanges second users Redeemable tokens for watermelon", async () => {
    secondUserWatermelon = await createTokenAccount(
      provider,
      watermelonMint,
      provider.wallet.publicKey
    );

    await program.rpc.exchangeRedeemableForWatermelon(secondDeposit, {
      accounts: {
        idoAccount: idoAccount.publicKey,
        poolSigner,
        redeemableMint,
        poolWatermelon,
        userAuthority: provider.wallet.publicKey,
        userWatermelon: secondUserWatermelon,
        userRedeemable: secondUserRedeemable,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
    });

    poolWatermelonAccount = await getTokenAccount(provider, poolWatermelon);
    assert.ok(poolWatermelonAccount.amount.eq(new anchor.BN(0)));
  });

  it("Withdraws total USDC from pool account", async () => {
    await program.rpc.withdrawPoolUsdc({
      accounts: {
        idoAccount: idoAccount.publicKey,
        poolSigner,
        idoAuthority: provider.wallet.publicKey,
        idoAuthorityUsdc,
        poolUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
    });

    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(new anchor.BN(0)));
    idoAuthorityUsdcAccount = await getTokenAccount(provider, idoAuthorityUsdc);
    assert.ok(idoAuthorityUsdcAccount.amount.eq(totalPoolUsdc));
  });

  function PoolBumps() {
    this.idoAccount;
    this.redeemableMint;
    this.poolWatermelon;
    this.poolUsdc;
  }
});
