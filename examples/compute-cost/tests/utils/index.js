// TODO: Modernize all these apis. This is all quite clunky.

const anchor = require("@project-serum/anchor");
const Token = require("@solana/spl-token").Token;
const TOKEN_PROGRAM_ID = require("@solana/spl-token").TOKEN_PROGRAM_ID;
const AccountLayout = require("@solana/spl-token").AccountLayout;
const u64 = require("@solana/spl-token").u64;
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const Market = require("@project-serum/serum").Market;
const DexInstructions = require("@project-serum/serum").DexInstructions;
const web3 = require("@project-serum/anchor").web3;
const BN = require("@project-serum/anchor").BN;
// const serumCmn = require("@project-serum/common");
const Account = web3.Account;
const Transaction = web3.Transaction;
const PublicKey = web3.PublicKey;
const SystemProgram = web3.SystemProgram;
const TransactionInstruction = web3.TransactionInstruction;
const DEX_PID = new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");


function shortnameToSeed(s) {
  return "foresight_"+s;
}

// Creates everything needed for an orderbook to be running
//
// * Mints for both the base and quote currencies.
// * Lists the market.
// * Provides resting orders on the market.
//
// Returns a client that can be used to interact with the market
// (and some other data, e.g., the mints and market maker account).
async function initOrderbook({
  provider,
  bids,
  asks,
  fUsdcMint,
  yesMint,
  serumMarket,
  GOD_A,
  GOD_USDC,
  decimals,
}) {
  if (!bids || !asks) {
  // Henry: [Price (0.001 smallest), Quantity (0.01 smallest)]

  // Setup A/USDC and B/USDC markets with resting orders.
  asks = [
    [0.745, 7.84],
    [0.735, 1.32],
    [0.712, 2.41],
  ];
  bids = [
    [0.702, 8.51],
    [0.699, 12.93],
    [0.688, 82.80],
  ];
  }
  const MINT_A = yesMint;
  const USDC = fUsdcMint;


  // Create a funded account to act as market maker.
  const amount = 10000000 * 10 ** decimals;
  const marketMaker = await fundAccount({
    provider,
    mints: [
      { god: GOD_A, mint: MINT_A, amount, decimals },
      { god: GOD_USDC, mint: USDC, amount, decimals },
    ],
  });

  marketClient = await setupMarket({
    baseMint: MINT_A,
    quoteMint: USDC,
    marketMaker: {
      account: marketMaker.account,
      baseToken: marketMaker.tokens[MINT_A.toString()],
      quoteToken: marketMaker.tokens[USDC.toString()],
    },
    bids,
    asks,
    provider,
    marketAPublicKey: serumMarket,
  });

  return {
    marketClient,
    baseMint: MINT_A,
    quoteMint: USDC,
    marketMaker,
    godYes: GOD_A,
    godfUsdc: GOD_USDC,
  };
}

async function fundAccount({ provider, mints }) {
  const MARKET_MAKER = new Account();

  const marketMaker = {
    tokens: {},
    account: MARKET_MAKER,
  };

  // Transfer lamports to market maker.
  await provider.send(
    (() => {
      const tx = new Transaction();
      tx.add(
        SystemProgram.transfer({
          fromPubkey: provider.wallet.publicKey,
          toPubkey: MARKET_MAKER.publicKey,
          lamports: 100000000000,
        })
      );
      return tx;
    })()
  );

  // Transfer SPL tokens to the market maker.
  for (let k = 0; k < mints.length; k += 1) {
    const { mint, god, amount, decimals } = mints[k];
    let MINT_A = mint;
    let GOD_A = god;
    // Setup token accounts owned by the market maker.
    const mintAClient = new Token(
      provider.connection,
      MINT_A,
      TOKEN_PROGRAM_ID,
      provider.wallet.payer // node only
    );
    const marketMakerTokenA = await mintAClient.createAccount(
      MARKET_MAKER.publicKey
    );

    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          Token.createTransferCheckedInstruction(
            TOKEN_PROGRAM_ID,
            GOD_A,
            MINT_A,
            marketMakerTokenA,
            provider.wallet.publicKey,
            [],
            amount,
            decimals
          )
        );
        return tx;
      })()
    );

    marketMaker.tokens[mint.toString()] = marketMakerTokenA;
  }

  return marketMaker;
}

async function setupMarket({
  marketAPublicKey,
  provider,
  marketMaker,
  baseMint,
  quoteMint,
  bids,
  asks,
}) {
  const MARKET_A_USDC = await Market.load(
    provider.connection,
    marketAPublicKey,
    { commitment: "recent" },
    DEX_PID
  );
  for (let k = 0; k < asks.length; k += 1) {
    let ask = asks[k];
    const {
      transaction,
      signers,
    } = await MARKET_A_USDC.makePlaceOrderTransaction(provider.connection, {
      owner: marketMaker.account,
      payer: marketMaker.baseToken,
      side: "sell",
      price: ask[0],
      size: ask[1],
      orderType: "postOnly",
      clientId: undefined, // todo?
      openOrdersAddressKey: undefined,
      openOrdersAccount: undefined,
      feeDiscountPubkey: null,
      selfTradeBehavior: "abortTransaction",
    });
    await provider.send(transaction, signers.concat(marketMaker.account));
  }

  for (let k = 0; k < bids.length; k += 1) {
    let bid = bids[k];
    const {
      transaction,
      signers,
    } = await MARKET_A_USDC.makePlaceOrderTransaction(provider.connection, {
      owner: marketMaker.account,
      payer: marketMaker.quoteToken,
      side: "buy",
      price: bid[0],
      size: bid[1],
      orderType: "postOnly",
      clientId: undefined, // todo?
      openOrdersAddressKey: undefined,
      openOrdersAccount: undefined,
      feeDiscountPubkey: null,
      selfTradeBehavior: "abortTransaction",
    });
    await provider.send(transaction, signers.concat(marketMaker.account));
  }

  return MARKET_A_USDC;
}

async function listMarket({
  connection,
  wallet,
  baseMint,
  quoteMint,
  baseLotSize,
  quoteLotSize,
  dexProgramId,
  feeRateBps,
}) {
  const market = new Account();
  const requestQueue = new Account();
  const eventQueue = new Account();
  const bids = new Account();
  const asks = new Account();
  const baseVault = new Account();
  const quoteVault = new Account();
  const quoteDustThreshold = new BN(100);

  async function getVaultOwnerAndNonce() {
    const nonce = new BN(0);
    while (true) {
      try {
        const vaultOwner = await PublicKey.createProgramAddress(
          [market.publicKey.toBuffer(), nonce.toArrayLike(Buffer, "le", 8)],
          dexProgramId
        );
        return [vaultOwner, nonce];
      } catch (e) {
        nonce.iaddn(1);
      }
    }
  }
  const [vaultOwner, vaultSignerNonce] = await getVaultOwnerAndNonce();

  const tx1 = new Transaction();
  tx1.add(
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: baseVault.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(165),
      space: 165,
      programId: TOKEN_PROGRAM_ID,
    }),
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: quoteVault.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(165),
      space: 165,
      programId: TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeAccount({
      account: baseVault.publicKey,
      mint: baseMint,
      owner: vaultOwner,
    }),
    TokenInstructions.initializeAccount({
      account: quoteVault.publicKey,
      mint: quoteMint,
      owner: vaultOwner,
    })
  );

  const tx2 = new Transaction();
  tx2.add(
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: market.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        Market.getLayout(dexProgramId).span
      ),
      space: Market.getLayout(dexProgramId).span,
      programId: dexProgramId,
    }),
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: requestQueue.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(5120 + 12),
      space: 5120 + 12,
      programId: dexProgramId,
    }),
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: eventQueue.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(262144 + 12),
      space: 262144 + 12,
      programId: dexProgramId,
    }),
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: bids.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(65536 + 12),
      space: 65536 + 12,
      programId: dexProgramId,
    }),
    SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: asks.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(65536 + 12),
      space: 65536 + 12,
      programId: dexProgramId,
    }),
    DexInstructions.initializeMarket({
      market: market.publicKey,
      requestQueue: requestQueue.publicKey,
      eventQueue: eventQueue.publicKey,
      bids: bids.publicKey,
      asks: asks.publicKey,
      baseVault: baseVault.publicKey,
      quoteVault: quoteVault.publicKey,
      baseMint,
      quoteMint,
      baseLotSize: new BN(baseLotSize),
      quoteLotSize: new BN(quoteLotSize),
      feeRateBps,
      vaultSignerNonce,
      quoteDustThreshold,
      programId: dexProgramId,
    })
  );

  const signedTransactions = await signTransactions({
    transactionsAndSigners: [
      { transaction: tx1, signers: [baseVault, quoteVault] },
      {
        transaction: tx2,
        signers: [market, requestQueue, eventQueue, bids, asks],
      },
    ],
    wallet,
    connection,
  });
  for (let signedTransaction of signedTransactions) {
    await sendAndConfirmRawTransaction(
      connection,
      signedTransaction.serialize()
    );
  }
  const acc = await connection.getAccountInfo(market.publicKey);

  return market.publicKey;
}

async function signTransactions({
  transactionsAndSigners,
  wallet,
  connection,
}) {
  const blockhash = (await connection.getRecentBlockhash("max")).blockhash;
  transactionsAndSigners.forEach(({ transaction, signers = [] }) => {
    transaction.recentBlockhash = blockhash;
    transaction.setSigners(
      wallet.publicKey,
      ...signers.map((s) => s.publicKey)
    );
    if (signers.length > 0) {
      transaction.partialSign(...signers);
    }
  });
  return await wallet.signAllTransactions(
    transactionsAndSigners.map(({ transaction }) => transaction)
  );
}

async function sendAndConfirmRawTransaction(
  connection,
  raw,
  commitment = "recent"
) {
  let tx = await connection.sendRawTransaction(raw, {
    skipPreflight: true,
  });
  return await connection.confirmTransaction(tx, commitment);
}


async function getVaultOwnerAndNonce(marketPublicKey, dexProgramId = DEX_PID) {
  const nonce = new BN(0);
  while (nonce.toNumber() < 255) {
    try {
      const vaultOwner = await PublicKey.createProgramAddress(
        [marketPublicKey.toBuffer(), nonce.toArrayLike(Buffer, "le", 8)],
        dexProgramId
      );
      return [vaultOwner, nonce];
    } catch (e) {
      nonce.iaddn(1);
    }
  }
  throw new Error("Unable to find nonce");
}


async function mintToAccount(
  provider,
  mint,
  destination,
  amount,
  mintAuthority
) {
  // mint authority is the provider
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createMintToAccountInstrs(
      mint,
      destination,
      amount,
      mintAuthority
    ))
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





async function setAuthority(
  provider,
  mint,
  newAuthority,
  mintAuthority
) {
  // mint authority is the provider
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createSetAuthorityInstrs(
      mint,
      mintAuthority,
      newAuthority,
    ))
  );
  await provider.send(tx, []);
  return;
}

async function createSetAuthorityInstrs(
  target,
  currentAuthority,
  newAuthority,
) {
  // This is the enum value for authority type 
  // https://github.com/solana-labs/solana-program-library/blob/master/token/program/src/instruction.rs#L583-L613
  const authorityType = new anchor.BN(0)
  return [
    TokenInstructions.setAuthority({
      target,
      currentAuthority,
      newAuthority,
      authorityType
    }),
  ];
}


async function printOrders(
  marketClient,
  provider
) {
      // Check the orders
    console.log("Price Quantity Side")
    // Fetching orderbooks
    let bids = await marketClient.loadBids(provider.connection);
    let asks = await marketClient.loadAsks(provider.connection);
    // Full orderbook data
    for (let order of asks) {
      console.log(
        // order.orderId,
        order.price,
        // order.priceLots.toNumber(),
        order.size,
        // order.sizeLots.toNumber(),
        order.side, // 'buy' or 'sell'
      );
    }
    // Full orderbook data
    // TODO how to print in reverse?
    for (let order of bids) {
      console.log(
        order.orderId,
        // order.orderId,
        order.price,
        // order.priceLots.toNumber(),
        order.size,
        // order.sizeLots.toNumber(),
        order.side, // 'buy' or 'sell''
      );
    }
}


async function createMint(
  provider,
  authority,
  decimals,
) {
  if (authority === undefined) {
    authority = provider.wallet.publicKey;
  }
  const mint = new Account();
  const instructions = await createMintInstructions(
    provider,
    authority,
    mint.publicKey,
    decimals,
  );

  const tx = new Transaction();
  tx.add(...instructions);

  await provider.send(tx, [mint]);

  return mint.publicKey;
}

async function createMintInstructions(
  provider,
  authority,
  mint,
  decimals,
) {
  let instructions = [
    SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: mint,
      space: 82,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
      programId: TokenInstructions.TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeMint({
      mint,
      decimals: decimals ?? 0,
      mintAuthority: authority,
    }),
  ];
  return instructions;
}


async function createTokenAccount(
  provider,
  mint,
  owner,
) {
  const vault = new Account();
  const tx = new Transaction();
  tx.add(
    ...(await createTokenAccountInstrs(provider, vault.publicKey, mint, owner)),
  );
  await provider.send(tx, [vault]);
  return vault.publicKey;
}

async function createTokenAccountInstrs(
  provider,
  newAccountPubkey,
  mint,
  owner,
  lamports,
) {
  if (lamports === undefined) {
    lamports = await provider.connection.getMinimumBalanceForRentExemption(165);
  }
  return [
    SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey,
      space: 165,
      lamports,
      programId: TokenInstructions.TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeAccount({
      account: newAccountPubkey,
      mint,
      owner,
    }),
  ];
}


async function getTokenAccount(
  provider,
  addr
) {
  let accInfo = await provider.connection.getAccountInfo(addr);
  if (accInfo === null) {
    throw new Error('Failed to find token account');
  }
  return parseTokenAccount(accInfo.data);
}

function parseTokenAccount(data) {
  const accountInfo = AccountLayout.decode(data);
  accountInfo.mint = new web3.PublicKey(accountInfo.mint);
  accountInfo.owner = new web3.PublicKey(accountInfo.owner);;
  accountInfo.amount = u64.fromBuffer(accountInfo.amount);
  return accountInfo;
}

module.exports = {
  fundAccount,
  setupMarket,
  initOrderbook,
  getVaultOwnerAndNonce,
  DEX_PID,
  mintToAccount,
  setAuthority,
  listMarket,
  printOrders,
  createMint,
  createTokenAccount,
  getTokenAccount,
};