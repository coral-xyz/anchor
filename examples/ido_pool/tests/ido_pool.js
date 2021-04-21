const anchor = require('@project-serum/anchor');
const assert = require('assert');


describe('ido_pool', () => {

  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.IdoPool;

  // All mints default to 6 decimal places
  const watermelonIdoAmount = new anchor.BN(5000000);

  // These are all of the variables we assume exist in the world already and
  // are available to the client.
  let usdcMint = null;
  let watermelonMint = null;
  let creatorUsdc = null;
  let creatorWatermelon = null;
  

  it('Initializes the state-of-the-world', async () => {
    usdcMint = await createMint(provider);
    watermelonMint = await createMint(provider);
    creatorUsdc =  await createTokenAccount(provider, usdcMint, provider.wallet.publicKey);
    creatorWatermelon =  await createTokenAccount(provider, watermelonMint, provider.wallet.publicKey);
    // Mint Watermelon tokens the will be distributed from the IDO pool
    await mintToAccount(provider, watermelonMint, creatorWatermelon, watermelonIdoAmount, provider.wallet.publicKey);
    creator_watermelon_account = await getTokenAccount(provider, creatorWatermelon);
    assert.ok(creator_watermelon_account.amount.eq(watermelonIdoAmount));
  });

    // console.log(Object.getOwnPropertyNames(TokenInstructions).filter(function (p) {
    //   return typeof TokenInstructions[p] === 'function';
    // }));

    // console.log(creators_watermelon_account.amount)
    // const tx = await program.rpc.initialize();
    // console.log('Your transaction signature', tx);


  // These are all variables the client will have to create to initialize the
  // IDO pool
  let poolSigner = null;
  let redeemableMint = null;
  let poolWatermelon = null;
  let poolUsdc = null;
  let poolAccount = null;


  it('Initializes the IDO pool', async () => {
    // We use the watermelon mint address as the seed, could use something else though
    const [
      _poolSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [watermelonMint.toBuffer()],
      program.programId
    );
    poolSigner = _poolSigner;

    // Pool doesn't need a Redeemable SPL token account because it only
    // burns and mints redeemable tokens, it never stores them
    redeemableMint = await createMint(provider, poolSigner);
    poolWatermelon =  await createTokenAccount(provider, watermelonMint, poolSigner);
    poolUsdc =  await createTokenAccount(provider, usdcMint, poolSigner);

    poolAccount = new anchor.web3.Account();
    // console.log(program);
    // console.log(program.account);
    
    // Atomically create the new account and initialize it with the program.
    await program.rpc.initializePool(watermelonIdoAmount, nonce, {
      accounts: {
        poolAccount: poolAccount.publicKey,
        poolSigner,
        distributionAuthority: provider.wallet.publicKey,
        creatorWatermelon,
        creatorUsdc,
        redeemableMint,
        poolWatermelon,
        poolUsdc,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [poolAccount],
      instructions: [
        await program.account.poolAccount.createInstruction(poolAccount),
      ],
    });

    creators_watermelon_account = await getTokenAccount(provider, creatorWatermelon);
    assert.ok(creators_watermelon_account.amount.eq(new anchor.BN(0)));
  });

  // This is how you get account sizes
  // console.log(program.account.poolAccount.size)

  // We're going to need to start using the associated program account for creating token accounts
  // if not in testing, then definitely in production
  
  let userUsdc = null;
  let userRedeemable = null;
  // 10 usdc
  const firstDeposit = new anchor.BN(10_000_000);

  it('Exchanges user USDC for redeemable tokens', async () => {
    userUsdc =  await createTokenAccount(provider, usdcMint, provider.wallet.publicKey);
    await mintToAccount(provider, usdcMint, userUsdc, firstDeposit, provider.wallet.publicKey);
    userRedeemable =  await createTokenAccount(provider, redeemableMint, provider.wallet.publicKey);

    await program.rpc.exchangeUsdcForRedeemable(firstDeposit, {
      accounts: {
        poolAccount: poolAccount.publicKey,
        poolSigner,
        redeemableMint,
        poolUsdc,
        userAuthority: provider.wallet.publicKey,
        userUsdc,
        userRedeemable,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });
    
    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    assert.ok(poolUsdcAccount.amount.eq(firstDeposit));
    userRedeemableAccount = await getTokenAccount(provider, userRedeemable);
    assert.ok(userRedeemableAccount.amount.eq(firstDeposit));

  });

  // 23 usdc
  const secondDeposit = new anchor.BN(23_000_000);
  let totalPoolUsdc = null;

  it('Exchanges a second users USDC for redeemable tokens', async () => {
    secondUserUsdc =  await createTokenAccount(provider, usdcMint, provider.wallet.publicKey);
    await mintToAccount(provider, usdcMint, secondUserUsdc, secondDeposit, provider.wallet.publicKey);
    secondUserRedeemable =  await createTokenAccount(provider, redeemableMint, provider.wallet.publicKey);

    await program.rpc.exchangeUsdcForRedeemable(secondDeposit, {
      accounts: {
        poolAccount: poolAccount.publicKey,
        poolSigner,
        redeemableMint,
        poolUsdc,
        userAuthority: provider.wallet.publicKey,
        userUsdc: secondUserUsdc,
        userRedeemable: secondUserRedeemable,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });
    
    totalPoolUsdc = firstDeposit.add(secondDeposit);
    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    // console.log("pool usdc: ", poolUsdcAccount.amount.toNumber(), "expected usdc:", totalPoolUsdc.toNumber());
    assert.ok(poolUsdcAccount.amount.eq(totalPoolUsdc));
    secondUserRedeemableAccount = await getTokenAccount(provider, secondUserRedeemable);
    assert.ok(secondUserRedeemableAccount.amount.eq(secondDeposit));
  });


  const firstWithdrawal = new anchor.BN(2_000_000);
  // console.log(firstDeposit.toNumber(), secondDeposit.toNumber(), firstWithdrawal.toNumber());
  // totalPoolUsdc = totalPoolUsdc.sub(firstWithdrawal);

  it('Exchanges user Redeemable tokens for USDC', async () => {
    await program.rpc.exchangeRedeemableForUsdc(firstWithdrawal, {
      accounts: {
        poolAccount: poolAccount.publicKey,
        poolSigner,
        redeemableMint,
        poolUsdc,
        userAuthority: provider.wallet.publicKey,
        userUsdc,
        userRedeemable,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });

    totalPoolUsdc = totalPoolUsdc.sub(firstWithdrawal);
    poolUsdcAccount = await getTokenAccount(provider, poolUsdc);
    // console.log("pool usdc: ", poolUsdcAccount.amount.toNumber(), "expected usdc:", totalPoolUsdc.toNumber());
    assert.ok(poolUsdcAccount.amount.eq(totalPoolUsdc));
    userUsdcAccount = await getTokenAccount(provider, userUsdc);
    assert.ok(userUsdcAccount.amount.eq(firstWithdrawal));
  });

  const ACCURACY = 6;

  it('Calculates the exchange rate', async () => {
    await program.rpc.calculateExchangeRate({
      accounts: {
        poolAccount: poolAccount.publicKey,
        redeemableMint,
        poolUsdc,
        poolWatermelon,
      },
    });

    let thisPoolAccount = await program.account.poolAccount(poolAccount.publicKey);
    // console.log("pool exchange rate: ", thisPoolAccount.exchangeRate);
    exchangeRate = watermelonIdoAmount.mul(new anchor.BN(10 ** ACCURACY));
    exchangeRate = exchangeRate.div(poolUsdcAccount.amount);
    // console.log("test exchange rate: ", exchangeRate);
    assert.ok(thisPoolAccount.exchangeRate.eq(exchangeRate));
  });


  // Got to do some calculations for how much should be left
  // const watermelonIdoAmount = new anchor.BN(5000000);

  it('Exchanges user Redeemable tokens for watermelon', async () => {
    let firstUserRedeemable = firstDeposit.sub(firstWithdrawal);
    userWatermelon =  await createTokenAccount(provider, watermelonMint, provider.wallet.publicKey);

    await program.rpc.exchangeRedeemableForWatermelon(firstUserRedeemable, {
      accounts: {
        poolAccount: poolAccount.publicKey,
        poolSigner,
        redeemableMint,
        poolWatermelon,
        userAuthority: provider.wallet.publicKey,
        userWatermelon,
        userRedeemable,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });

    poolWatermelonAccount = await getTokenAccount(provider, poolWatermelon);
    // Should have idoAmount - (exchange rate * remaining amount)/10**ACCURACY
    let redeemedWatermelon = (exchangeRate.mul(firstUserRedeemable)).div(new anchor.BN(10 ** ACCURACY));
    let remainingWatermelon = watermelonIdoAmount.sub(redeemedWatermelon);
    assert.ok(poolWatermelonAccount.amount.eq(remainingWatermelon));
    userWatermelonAccount = await getTokenAccount(provider, userWatermelon);
    // Should have exchange rate * remaining amount / 10**ACCURACY
    assert.ok(userWatermelonAccount.amount.eq(redeemedWatermelon));
  });

});





// SPL token client boilerplate for test initialization. Everything below here is
// mostly irrelevant to the point of the example.

const serumCmn = require('@project-serum/common');
const TokenInstructions = require('@project-serum/serum').TokenInstructions;

// TODO: remove this constant once @project-serum/serum uses the same version
//       of @solana/web3.js as anchor (or switch packages).
const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
  TokenInstructions.TOKEN_PROGRAM_ID.toString()
);

async function getTokenAccount(provider, addr) {
  return await serumCmn.getTokenAccount(provider, addr);
}

async function createMint(provider, authority) {
  if (authority === undefined) {
    authority = provider.wallet.publicKey;
  }
  const mint = new anchor.web3.Account();
  const instructions = await createMintInstructions(
    provider,
    authority,
    mint.publicKey
  );

  const tx = new anchor.web3.Transaction();
  tx.add(...instructions);

  await provider.send(tx, [mint]);

  return mint.publicKey;
}

async function createMintInstructions(provider, authority, mint) {
  let instructions = [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: mint,
      space: 82,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
      programId: TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeMint({
      mint,
      decimals: 6,
      mintAuthority: authority,
    }),
  ];
  return instructions;
}

async function createTokenAccount(provider, mint, owner) {
  const vault = new anchor.web3.Account();
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createTokenAccountInstrs(provider, vault.publicKey, mint, owner))
  );
  await provider.send(tx, [vault]);
  return vault.publicKey;
}

async function createTokenAccountInstrs(
  provider,
  newAccountPubkey,
  mint,
  owner,
  lamports
) {
  if (lamports === undefined) {
    lamports = await provider.connection.getMinimumBalanceForRentExemption(165);
  }
  return [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey,
      space: 165,
      lamports,
      programId: TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeAccount({
      account: newAccountPubkey,
      mint,
      owner,
    }),
  ];
}


async function mintToAccount(provider, mint, destination, amount, mintAuthority) {
  // mint authority is the provider
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createMintToAccountInstrs(mint, destination, amount, mintAuthority))
  );
  await provider.send(tx, []);
  return;
}

async function createMintToAccountInstrs(
  mint,
  destination,
  amount,
  mintAuthority
) {
  return [
    TokenInstructions.mintTo({
      mint,
      destination: destination,
      amount: amount,
      mintAuthority: mintAuthority,
    }),
  ];
}
