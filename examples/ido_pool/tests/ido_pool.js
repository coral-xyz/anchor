const anchor = require('@project-serum/anchor');
const assert = require('assert');


describe('ido_pool', () => {

  const provider = anchor.Provider.local();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.IdoPool;
  const watermelon_ido_amount = new anchor.BN(500)

  let usdc_mint = null;
  let watermelon_mint = null;


  it('Initializes the state-of-the-world', async () => {
    usdc_mint = await createMint(provider);
    watermelon_mint = await createMint(provider);
    creators_watermelon_publickey =  await createTokenAccount(provider, watermelon_mint, provider.wallet.publicKey);
    // Tokens to distributed from the IDO pool
    await mintToAccount(provider, watermelon_mint, creators_watermelon_publickey, watermelon_ido_amount, provider.wallet.publicKey);
    creators_watermelon_account = await getTokenAccount(provider, creators_watermelon_publickey);
    assert.ok(creators_watermelon_account.amount.eq(watermelon_ido_amount));
    // console.log(Object.getOwnPropertyNames(TokenInstructions).filter(function (p) {
    //   return typeof TokenInstructions[p] === 'function';
    // }));

    // console.log(creators_watermelon_account.amount)
    // const tx = await program.rpc.initialize();
    // console.log('Your transaction signature', tx);
  });



  let poolSigner = null;
  let pool_watermelon_publickey = null
  let pool_usdc_publickey = null


  it('Initializes the IDO pool', async () => {
    // We use the watermelon mint address as the seed, could use something else though
    const [
      _poolSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [watermelon_mint.toBuffer()],
      program.programId
    );
    poolSigner = _poolSigner;
    // console.log(poolSigner);

    pool_watermelon_publickey =  await createTokenAccount(provider, watermelon_mint, poolSigner);
    pool_usdc_publickey =  await createTokenAccount(provider, usdc_mint, poolSigner);




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
      decimals: 0,
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