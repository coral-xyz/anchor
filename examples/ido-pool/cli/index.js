const anchor = require("@project-serum/anchor");
const serum = require("@project-serum/common");
const yargs = require('yargs/yargs');
const { hideBin } = require('yargs/helpers')
const { TokenInstructions } = require("@project-serum/serum");

const provider = anchor.Provider.local(process.env.CLUSTER_RPC_URL);
// Configure the client to use the local cluster.
anchor.setProvider(provider);

const program = anchor.workspace.IdoPool;

// TODO: remove this constant once @project-serum/serum uses the same version
//       of @solana/web3.js as anchor (or switch packages).
const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
  TokenInstructions.TOKEN_PROGRAM_ID.toString()
);


async function createMint(provider, authority, decimals=9) {
  if (authority === undefined) {
    authority = provider.wallet.publicKey;
  }
  const mint = new anchor.web3.Account();
  const instructions = await createMintInstructions(
    provider,
    authority,
    mint.publicKey,
    decimals
  );

  const tx = new anchor.web3.Transaction();
  tx.add(...instructions);

  await provider.send(tx, [mint]);

  return mint.publicKey;
}

async function createMintInstructions(provider, authority, mint, decimals) {
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
      decimals,
      mintAuthority: authority,
    }),
  ];
  return instructions;
}

async function createTokenAccount(provider, mint, owner) {
  const vault = new anchor.web3.Account();
  const tx = new anchor.web3.Transaction();
  console.log('createTokenAccount', vault.publicKey.toString(), mint.toString(), owner.toString());

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

async function initPool(
  usdcMint, watermelonMint, creatorWatermelon, watermelonIdoAmount,
  startIdoTs, endDepositsTs, endIdoTs) {

  // We use the watermelon mint address as the seed, could use something else though.
  const [_poolSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [watermelonMint.toBuffer()],
    program.programId
  );
  poolSigner = _poolSigner;

  // fetch usdc mint to set redeemable decimals to the same value
  const mintInfo = await serum.getMintInfo(provider, usdcMint)

  // Pool doesn't need a Redeemable SPL token account because it only
  // burns and mints redeemable tokens, it never stores them.
  redeemableMint = await createMint(provider, poolSigner, mintInfo.decimals);
  poolWatermelon = await createTokenAccount(provider, watermelonMint, poolSigner);
  poolUsdc = await createTokenAccount(provider, usdcMint, poolSigner);
  poolAccount = new anchor.web3.Account();
  distributionAuthority = provider.wallet.publicKey;


  console.log('initializePool', watermelonIdoAmount.toString(), nonce, startIdoTs.toString(), endDepositsTs.toString(), endIdoTs.toString())
  // Atomically create the new account and initialize it with the program.
  await program.rpc.initializePool(
    watermelonIdoAmount,
    nonce,
    startIdoTs,
    endDepositsTs,
    endIdoTs,
    {
      accounts: {
        poolAccount: poolAccount.publicKey,
        poolSigner,
        distributionAuthority,
        creatorWatermelon,
        redeemableMint,
        usdcMint,
        poolWatermelon,
        poolUsdc,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
      signers: [poolAccount],
      instructions: [
        await program.account.poolAccount.createInstruction(poolAccount),
      ],
    }
  );

  console.log(`🏦 IDO pool initialized with ${(watermelonIdoAmount.toNumber() / 1000000).toFixed(2)} tokens`);
  console.log(`Pool Account: ${poolAccount.publicKey.toBase58()}`);
  console.log(`Pool Authority: ${distributionAuthority.toBase58()}`);
  console.log(`Redeem Mint: ${redeemableMint.toBase58()}`);
  console.log(`🍉 Account: ${poolWatermelon.toBase58()}`);
  console.log(`💵 Account: ${poolUsdc.toBase58()}`);
}


async function bid(poolAccount, userUsdc, bidAmount, userRedeemable) {

  const account = await program.account.poolAccount.fetch(poolAccount);

  // We use the watermelon mint address as the seed, could use something else though.
  const [_poolSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [account.watermelonMint.toBuffer()],
    program.programId
  );
  poolSigner = _poolSigner;

  const currentBid = await serum.getTokenAccount(provider, userRedeemable);

  if (currentBid.amount.lt(bidAmount)) {
    const depositAmount = bidAmount.sub(currentBid.amount);
    console.log(`increasing bid by ${(depositAmount.toNumber() / 1000000).toFixed(2)} 💵`);

    await program.rpc.exchangeUsdcForRedeemable(
      depositAmount,
      {
        accounts: {
          poolAccount,
          poolSigner,
          redeemableMint: account.redeemableMint,
          poolUsdc: account.poolUsdc,
          userAuthority: provider.wallet.publicKey,
          userUsdc,
          userRedeemable,
          tokenProgram: TOKEN_PROGRAM_ID,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      });
  } else if (currentBid.amount.gt(bidAmount)) {
    const withdrawAmount = currentBid.amount.sub(bidAmount);
    console.log(`decreasing bid by ${(withdrawAmount.toNumber() / 1000000).toFixed(2)} 💵`);

    await program.rpc.exchangeRedeemableForUsdc(withdrawAmount, {
      accounts: {
        poolAccount,
        poolSigner,
        redeemableMint: account.redeemableMint,
        poolUsdc: account.poolUsdc,
        userAuthority: provider.wallet.publicKey,
        userUsdc,
        userRedeemable,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
    });

  } else {
    console.log('bid unchanged 💎');
  }
}

const usdc_mint = {
  describe: 'the mint of the token sale bids 💵',
  type: 'string'
}

const watermelon_mint = {
  describe: 'the mint of the token for sale 🍉',
  type: 'string'
}

const watermelon_account = {
  describe: 'the account supplying the token for sale 🍉',
  type: 'string'
}

const watermelon_amount = {
  describe: 'the amount of tokens offered in this sale 🍉',
  type: 'number'
}

const pool_account = {
  describe: 'the token sale pool account 🏦',
  type: 'string'
}

const start_time = {
  describe: 'the unix time at which the token sale is starting',
  default: 10 + (Date.now() / 1000),
  type: 'number'
}

const deposit_duration = {
  describe: 'the number of seconds users can deposit into the pool',
  default: 24 * 60 * 60,
  type: 'number'
}

const cancel_duration = {
  describe: 'the number of seconds users can withdraw from the pool to cancel their bid',
  default: 24 * 60 * 60,
  type: 'number'
}


yargs(hideBin(process.argv))
  .command(
    'init <usdc_mint> <watermelon_mint> <watermelon_account> <watermelon_amount>',
    'initialize IDO pool',
    y => y
      .positional('usdc_mint', usdc_mint)
      .positional('watermelon_mint', watermelon_mint)
      .positional('watermelon_account', { describe: 'the account supplying the token for sale 🍉', type: 'string' })
      .positional('watermelon_amount', { describe: 'the amount of tokens offered in this sale 🍉', type: 'number' })
      .option('start_time', start_time)
      .option('deposit_duration', deposit_duration)
      .option('cancel_duration', cancel_duration),
    args => {
      const start = new anchor.BN(args.start_time);
      const endDeposits = new anchor.BN(args.deposit_duration).add(start);
      const endIdo = new anchor.BN(args.cancel_duration).add(endDeposits);
      initPool(
        new anchor.web3.PublicKey(args.usdc_mint),
        new anchor.web3.PublicKey(args.watermelon_mint),
        new anchor.web3.PublicKey(args.watermelon_account),
        new anchor.BN(args.watermelon_amount * 1000000), // assuming 6 decimals
        start,
        endDeposits,
        endIdo
      );
    })
  .command(
    'bid <pool_account> <usdc_account> <usdc_amount> <redeemable_account>',
    'place bid in IDO sale',
     y => y
      .positional('pool_account', pool_account)
      .positional('usdc_account', { describe: 'the account supplying the token sale bids 💵', type: 'string' })
      .positional('usdc_amount', { describe: 'the amount of tokens bid for this sale 💵', type: 'number' })
      .positional('redeemable_account', { describe: 'the account receiving the redeemable pool token', type: 'string' }),
    args => {
      bid(
        new anchor.web3.PublicKey(args.pool_account),
        new anchor.web3.PublicKey(args.usdc_account),
        new anchor.BN(args.usdc_amount * 1000000), // assuming 6 decimals
        new anchor.web3.PublicKey(args.redeemable_account)
      );
    })
  .argv;
