const assert = require("assert");
const { Token } = require("@solana/spl-token");
const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
const { Market } = require("@project-serum/serum");
const utf8 = anchor.utils.bytes.utf8;
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } = anchor.web3;
const utils = require("./utils");
const { setupStakePool } = require("./utils/stake");

const DEX_PID = new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");
const SWAP_PID = new PublicKey("22Y43yTVxuUkoRKdm9thyRhQ3SdgQS7c7kB6UNCiaczD");
const TOKEN_PID = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const REGISTRY_PID = new PublicKey(
  "GrAkKfEpTKQuVHG2Y97Y2FF4i7y7Q5AHLK94JBy7Y5yv"
);
const LOCKUP_PID = new PublicKey(
  "6ebQNeTPZ1j7k3TtkCCtEPRvG7GQsucQrZ7sSEDQi9Ks"
);
const SYSVAR_INSTRUCTIONS_PUBKEY = new PublicKey(
  "Sysvar1nstructions1111111111111111111111111"
);
const FEES = "6160355581";

describe("cfo", () => {
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Cfo;
  const sweepAuthority = program.provider.wallet.publicKey;
  let officer, srmVault, usdcVault, bVault, stake, treasury;
  let officerBump, srmBump, usdcBump, bBump, stakeBump, treasuryBump;
  let openOrders, openOrdersBump;
  let USDC_TOKEN_CLIENT, A_TOKEN_CLIENT, B_TOKEN_CLIENT;
  let officerAccount;
  let marketAClient;

  // Accounts used to setup the orderbook.
  let ORDERBOOK_ENV,
    // Accounts used for A -> USDC swap transactions.
    SWAP_A_USDC_ACCOUNTS,
    // Accounts used for  USDC -> A swap transactions.
    SWAP_USDC_A_ACCOUNTS,
    // Serum DEX vault PDA for market A/USDC.
    marketAVaultSigner,
    // Serum DEX vault PDA for market B/USDC.
    marketBVaultSigner;

  let registrar, msrmRegistrar;

  it("BOILERPLATE: Sets up a market with funded fees", async () => {
    ORDERBOOK_ENV = await utils.initMarket({
      provider: program.provider,
    });
    console.log("Token A: ", ORDERBOOK_ENV.marketA.baseMintAddress.toString());
    console.log(
      "Token USDC: ",
      ORDERBOOK_ENV.marketA.quoteMintAddress.toString()
    );
    USDC_TOKEN_CLIENT = new Token(
      program.provider.connection,
      ORDERBOOK_ENV.usdc,
      TOKEN_PID,
      program.provider.wallet.payer
    );
    SRM_TOKEN_CLIENT = new Token(
      program.provider.connection,
      ORDERBOOK_ENV.mintA,
      TOKEN_PID,
      program.provider.wallet.payer
    );
    B_TOKEN_CLIENT = new Token(
      program.provider.connection,
      ORDERBOOK_ENV.mintB,
      TOKEN_PID,
      program.provider.wallet.payer
    );

    await USDC_TOKEN_CLIENT.transfer(
      ORDERBOOK_ENV.godUsdc,
      ORDERBOOK_ENV.marketA._decoded.quoteVault,
      program.provider.wallet.payer,
      [],
      10000000000000
    );

    const tokenAccount = await USDC_TOKEN_CLIENT.getAccountInfo(
      ORDERBOOK_ENV.marketA._decoded.quoteVault
    );
    assert.ok(tokenAccount.amount.toString() === "10000902263700");
  });

  it("BOILERPLATE: Executes trades to generate fees", async () => {
    await utils.runTradeBot(
      ORDERBOOK_ENV.marketA._decoded.ownAddress,
      program.provider,
      1
    );
    marketAClient = await Market.load(
      program.provider.connection,
      ORDERBOOK_ENV.marketA._decoded.ownAddress,
      { commitment: "recent" },
      DEX_PID
    );
    assert.ok(marketAClient._decoded.quoteFeesAccrued.toString() === FEES);
  });

  it("BOILERPLATE: Sets up the staking pools", async () => {
    await setupStakePool(ORDERBOOK_ENV.mintA, ORDERBOOK_ENV.godA);
    registrar = ORDERBOOK_ENV.usdc;
    msrmRegistrar = registrar;
  });

  it("BOILERPLATE: Finds PDA addresses", async () => {
    const [_officer, _officerBump] = await PublicKey.findProgramAddress(
      [DEX_PID.toBuffer()],
      program.programId
    );
    const [_openOrders, _openOrdersBump] = await PublicKey.findProgramAddress(
      [utf8.encode("open-orders"), _officer.toBuffer()],
      program.programId
    );
    const [_srmVault, _srmBump] = await PublicKey.findProgramAddress(
      [
        utf8.encode("token"),
        _officer.toBuffer(),
        ORDERBOOK_ENV.mintA.toBuffer(),
      ],
      program.programId
    );
    const [_bVault, _bBump] = await PublicKey.findProgramAddress(
      [
        utf8.encode("token"),
        _officer.toBuffer(),
        ORDERBOOK_ENV.mintB.toBuffer(),
      ],
      program.programId
    );
    const [_usdcVault, _usdcBump] = await PublicKey.findProgramAddress(
      [
        utf8.encode("token"),
        _officer.toBuffer(),
        ORDERBOOK_ENV.usdc.toBuffer(),
      ],
      program.programId
    );
    const [_stake, _stakeBump] = await PublicKey.findProgramAddress(
      [utf8.encode("stake"), _officer.toBuffer()],
      program.programId
    );
    const [_treasury, _treasuryBump] = await PublicKey.findProgramAddress(
      [utf8.encode("treasury"), _officer.toBuffer()],
      program.programId
    );

    officer = _officer;
    officerBump = _officerBump;
    openOrders = _openOrders;
    openOrdersBump = _openOrdersBump;
    srmVault = _srmVault;
    srmBump = _srmBump;
    usdcVault = _usdcVault;
    usdcBump = _usdcBump;
    officer = _officer;
    officerBump = _officerBump;
    bVault = _bVault;
    bBump = _bBump;
    stake = _stake;
    stakeBump = _stakeBump;
    treasury = _treasury;
    treasuryBump = _treasuryBump;
  });

  it("Creates a CFO!", async () => {
    let distribution = {
      burn: 80,
      stake: 20,
      treasury: 0,
    };
    const bumps = {
      bump: officerBump,
      srm: srmBump,
      usdc: usdcBump,
      stake: stakeBump,
      treasury: treasuryBump,
    };
    await program.rpc.createOfficer(
      bumps,
      distribution,
      registrar,
      msrmRegistrar,
      {
        accounts: {
          officer,
          srmVault,
          usdcVault,
          stake,
          treasury,
          srmMint: ORDERBOOK_ENV.mintA,
          usdcMint: ORDERBOOK_ENV.usdc,
          authority: program.provider.wallet.publicKey,
          dexProgram: DEX_PID,
          swapProgram: SWAP_PID,
          tokenProgram: TOKEN_PID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        },
      }
    );

    officerAccount = await program.account.officer.fetch(officer);
    assert.ok(
      officerAccount.authority.equals(program.provider.wallet.publicKey)
    );
    assert.ok(
      JSON.stringify(officerAccount.distribution) ===
        JSON.stringify(distribution)
    );
  });

  it("Creates a token account for the officer associated with the market", async () => {
    await program.rpc.createOfficerToken(bBump, {
      accounts: {
        officer,
        token: bVault,
        mint: ORDERBOOK_ENV.mintB,
        payer: program.provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PID,
        rent: SYSVAR_RENT_PUBKEY,
      },
    });
    const tokenAccount = await B_TOKEN_CLIENT.getAccountInfo(bVault);
    assert.ok(tokenAccount.state === 1);
    assert.ok(tokenAccount.isInitialized);
  });

  it("Creates an open orders account for the officer", async () => {
    await program.rpc.createOfficerOpenOrders(openOrdersBump, {
      accounts: {
        officer,
        openOrders,
        payer: program.provider.wallet.publicKey,
        dexProgram: DEX_PID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        market: ORDERBOOK_ENV.marketA._decoded.ownAddress,
      },
    });
  });

  it("Sweeps fees", async () => {
    const [sweepVault, bump] = await PublicKey.findProgramAddress(
      [utf8.encode("token"), officer.toBuffer(), ORDERBOOK_ENV.usdc.toBuffer()],
      program.programId
    );
    const beforeTokenAccount = await serumCmn.getTokenAccount(
      program.provider,
      sweepVault
    );
    await program.rpc.sweepFees({
      accounts: {
        officer,
        sweepVault,
        mint: ORDERBOOK_ENV.usdc,
        dex: {
          market: ORDERBOOK_ENV.marketA._decoded.ownAddress,
          pcVault: ORDERBOOK_ENV.marketA._decoded.quoteVault,
          sweepAuthority,
          vaultSigner: ORDERBOOK_ENV.vaultSigner,
          dexProgram: DEX_PID,
          tokenProgram: TOKEN_PID,
        },
      },
    });
    const afterTokenAccount = await serumCmn.getTokenAccount(
      program.provider,
      sweepVault
    );
    assert.ok(
      afterTokenAccount.amount.sub(beforeTokenAccount.amount).toString() ===
        FEES
    );
  });

  it("Swap to SRM", async () => {
    const minExchangeRate = {
      rate: 0,
      fromDecimals: 6,
      quoteDecimals: 6,
      strict: false,
    };
    await program.rpc.swapToSrm(minExchangeRate, {
      accounts: {
        officer,
        market: {
          market: marketAClient.address,
          openOrders: marketAClient.decoded.openOrders,
          requestQueue: marketAClient.decoded.requestQueue,
          eventQueue: marketAClient.decoded.eventQueue,
          bids: marketAClient.decoded.bids,
          asks: marketAClient.decoded.asks,
          orderPayerTokenAccount: usdcVault,
          coinVault: marketAClient.decoded.baseVault,
          pcVault: marketAClient.decoded.quoteVault,
          vaultSigner: ORDERBOOK_ENV.vaultSigner,
        },
        usdcVault,
        srmVault,
        swapProgram: SWAP_PID,
        dexprogram: DEX_PID,
        tokenProgram: TOKEN_PID,
        instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        rent: SYSVAR_RENT_PUBKEY,
      },
    });
  });
});
