import * as assert from 'assert';
import * as anchor from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
import { Connection, ConfirmOptions, PublicKey } from '@solana/web3.js';
import * as BufferLayout from 'buffer-layout';

// import * as dotenv from 'dotenv';
// dotenv.config();

const CurveType = Object.freeze({
  ConstantProduct: 0, // Constant product curve, Uniswap-style
  ConstantPrice: 1, // Constant price curve, always X amount of A token for 1 B token, where X is defined at init
  Offset: 3, // Offset curve, like Uniswap, but with an additional offset on the token B side
});

describe("amm", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Amm;

  let authority: PublicKey;
  let bumpSeed: number;
  let tokenPool: Token;
  let tokenAccountPool: PublicKey;
  let feeAccount: PublicKey;
  const SWAP_PROGRAM_OWNER_FEE_ADDRESS = process.env.SWAP_PROGRAM_OWNER_FEE_ADDRESS;
  let mintA: Token;
  let mintB: Token;
  let tokenAccountA: PublicKey;
  let tokenAccountB: PublicKey;

  // Pool fees
  const TRADING_FEE_NUMERATOR = 25;
  const TRADING_FEE_DENOMINATOR = 10000;
  const OWNER_TRADING_FEE_NUMERATOR = 5;
  const OWNER_TRADING_FEE_DENOMINATOR = 10000;
  const OWNER_WITHDRAW_FEE_NUMERATOR = SWAP_PROGRAM_OWNER_FEE_ADDRESS ? 0 : 1;
  const OWNER_WITHDRAW_FEE_DENOMINATOR = SWAP_PROGRAM_OWNER_FEE_ADDRESS ? 0 : 6;
  const HOST_FEE_NUMERATOR = 20;
  const HOST_FEE_DENOMINATOR = 100;

  // Initial amount in each swap token
  let currentSwapTokenA = 1000000;
  let currentSwapTokenB = 1000000;
  let currentFeeAmount = 0;

  const SWAP_AMOUNT_IN = 100000;
  const SWAP_AMOUNT_OUT = SWAP_PROGRAM_OWNER_FEE_ADDRESS ? 90661 : 90674;
  const SWAP_FEE = SWAP_PROGRAM_OWNER_FEE_ADDRESS ? 22273 : 22277;
  const HOST_SWAP_FEE = SWAP_PROGRAM_OWNER_FEE_ADDRESS
    ? Math.floor((SWAP_FEE * HOST_FEE_NUMERATOR) / HOST_FEE_DENOMINATOR)
    : 0;
  const OWNER_SWAP_FEE = SWAP_FEE - HOST_SWAP_FEE;
  // Pool token amount minted on init
  const DEFAULT_POOL_TOKEN_AMOUNT = 1000000000;
  // Pool token amount to withdraw / deposit
  const POOL_TOKEN_AMOUNT = 10000000;

  const ammAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const owner = anchor.web3.Keypair.generate();

  it("Initialize AMM", async () => {

    const sig = await provider.connection.requestAirdrop(payer.publicKey, 10000000000);
    await provider.connection.confirmTransaction(
      sig,
      "singleGossip"
    );

    [authority, bumpSeed] = await PublicKey.findProgramAddress(
      [ammAccount.publicKey.toBuffer()],
      program.programId,
    );

    // creating pool mint

    tokenPool = await Token.createMint(
      provider.connection,
      payer,
      authority,
      null,
      2,
      TOKEN_PROGRAM_ID
    );

    // creating pool account
    tokenAccountPool = await tokenPool.createAccount(owner.publicKey);
    const ownerKey = SWAP_PROGRAM_OWNER_FEE_ADDRESS || owner.publicKey.toString();
    feeAccount = await tokenPool.createAccount(new PublicKey(ownerKey));

    // creating token A
    mintA = await Token.createMint(
      provider.connection,
      payer,
      owner.publicKey,
      null,
      2,
      TOKEN_PROGRAM_ID,
    );

    // creating token A account
    tokenAccountA = await mintA.createAccount(authority);
    // minting token A to swap
    await mintA.mintTo(tokenAccountA, owner, [], currentSwapTokenA);

    // creating token B
    mintB = await Token.createMint(
      provider.connection,
      payer,
      owner.publicKey,
      null,
      2,
      TOKEN_PROGRAM_ID,
    );

    // creating token B account
    tokenAccountB = await mintB.createAccount(authority);
    // minting token B to swap
    await mintB.mintTo(tokenAccountB, owner, [], currentSwapTokenB);

    const commandDataLayout = BufferLayout.struct([
      BufferLayout.nu64('tradeFeeNumerator'),
      BufferLayout.nu64('tradeFeeDenominator'),
      BufferLayout.nu64('ownerTradeFeeNumerator'),
      BufferLayout.nu64('ownerTradeFeeDenominator'),
      BufferLayout.nu64('ownerWithdrawFeeNumerator'),
      BufferLayout.nu64('ownerWithdrawFeeDenominator'),
      BufferLayout.nu64('hostFeeNumerator'),
      BufferLayout.nu64('hostFeeDenominator'),
      BufferLayout.u8('curveType'),
      BufferLayout.nu64('curveParameters'),
      // BufferLayout.blob(32, 'curveParameters'),
    ]);
    let data = Buffer.alloc(1024);
    const encodeLength = commandDataLayout.encode(
      {
        tradeFeeNumerator: TRADING_FEE_NUMERATOR,
        tradeFeeDenominator: TRADING_FEE_DENOMINATOR,
        ownerTradeFeeNumerator: OWNER_TRADING_FEE_NUMERATOR,
        ownerTradeFeeDenominator: OWNER_TRADING_FEE_DENOMINATOR,
        ownerWithdrawFeeNumerator: OWNER_WITHDRAW_FEE_NUMERATOR,
        ownerWithdrawFeeDenominator: OWNER_WITHDRAW_FEE_DENOMINATOR,
        hostFeeNumerator: HOST_FEE_NUMERATOR,
        hostFeeDenominator: HOST_FEE_DENOMINATOR,
        curveType: CurveType.ConstantProduct,
        curveParameters: 0,
      },
      data,
    );
    data = data.slice(0, encodeLength);
    const fees_input = {
      tradeFeeNumerator: new anchor.BN(TRADING_FEE_NUMERATOR),
      tradeFeeDenominator: new anchor.BN(TRADING_FEE_DENOMINATOR),
      ownerTradeFeeNumerator: new anchor.BN(OWNER_TRADING_FEE_NUMERATOR),
      ownerTradeFeeDenominator: new anchor.BN(OWNER_TRADING_FEE_DENOMINATOR),
      ownerWithdrawFeeNumerator: new anchor.BN(OWNER_WITHDRAW_FEE_NUMERATOR),
      ownerWithdrawFeeDenominator: new anchor.BN(OWNER_WITHDRAW_FEE_DENOMINATOR),
      hostFeeNumerator: new anchor.BN(HOST_FEE_NUMERATOR),
      hostFeeDenominator: new anchor.BN(HOST_FEE_DENOMINATOR),
    };
    const curve_input = {
      curveType: new anchor.BN(CurveType.ConstantProduct),
      curveParameters: new anchor.BN(0),
    };

    await program.rpc.initialize(
      fees_input,
      curve_input,
      {
        accounts: {
          authority: authority,
          amm: ammAccount.publicKey,
          tokenA: tokenAccountA,
          tokenB: tokenAccountB,
          poolMint: tokenPool.publicKey,
          feeAccount: feeAccount,
          destination: tokenAccountPool,
          tokenProgram: TOKEN_PROGRAM_ID
        },
        instructions: [
          await program.account.amm.createInstruction(ammAccount),
        ],
        signers: [ammAccount],
      }
    );

    let fetchedAmmAccount = await program.account.amm.fetch(
      ammAccount.publicKey
    );

    assert(fetchedAmmAccount.tokenProgramId.equals(TOKEN_PROGRAM_ID));
    assert(fetchedAmmAccount.tokenAAccount.equals(tokenAccountA));
    assert(fetchedAmmAccount.tokenBAccount.equals(tokenAccountB));
    assert(fetchedAmmAccount.tokenAMint.equals(mintA.publicKey));
    assert(fetchedAmmAccount.tokenBMint.equals(mintB.publicKey));
    assert(fetchedAmmAccount.poolMint.equals(tokenPool.publicKey));
    assert(fetchedAmmAccount.poolFeeAccount.equals(feeAccount));
    assert(
      TRADING_FEE_NUMERATOR == fetchedAmmAccount.fees.tradeFeeNumerator.toNumber(),
    );
    assert(
      TRADING_FEE_DENOMINATOR == fetchedAmmAccount.fees.tradeFeeDenominator.toNumber(),
    );
    assert(
      OWNER_TRADING_FEE_NUMERATOR ==
        fetchedAmmAccount.fees.ownerTradeFeeNumerator.toNumber(),
    );
    assert(
      OWNER_TRADING_FEE_DENOMINATOR ==
        fetchedAmmAccount.fees.ownerTradeFeeDenominator.toNumber(),
    );
    assert(
      OWNER_WITHDRAW_FEE_NUMERATOR ==
        fetchedAmmAccount.fees.ownerWithdrawFeeNumerator.toNumber(),
    );
    assert(
      OWNER_WITHDRAW_FEE_DENOMINATOR ==
        fetchedAmmAccount.fees.ownerWithdrawFeeDenominator.toNumber(),
    );
    assert(HOST_FEE_NUMERATOR == fetchedAmmAccount.fees.hostFeeNumerator.toNumber());
    assert(
      HOST_FEE_DENOMINATOR == fetchedAmmAccount.fees.hostFeeDenominator.toNumber(),
    );
    assert(curve_input.curveType == fetchedAmmAccount.curve.curveType);
  });

  it("DepositAllTokenTypes", async () => {
    const poolMintInfo = await tokenPool.getMintInfo();
    const supply = (poolMintInfo.supply as anchor.BN).toNumber();
    const swapTokenA = await mintA.getAccountInfo(tokenAccountA);
    const tokenAAmount = Math.floor(
      ((swapTokenA.amount as anchor.BN).toNumber() * POOL_TOKEN_AMOUNT) / supply,
    );
    const swapTokenB = await mintB.getAccountInfo(tokenAccountB);
    const tokenBAmount = Math.floor(
      ((swapTokenB.amount as anchor.BN).toNumber() * POOL_TOKEN_AMOUNT) / supply,
    );

    const userTransferAuthority = anchor.web3.Keypair.generate();
    // Creating depositor token a account
    const userAccountA = await mintA.createAccount(owner.publicKey);
    await mintA.mintTo(userAccountA, owner, [], tokenAAmount);
    await mintA.approve(
      userAccountA,
      userTransferAuthority.publicKey,
      owner,
      [],
      tokenAAmount,
    );
    // Creating depositor token b account
    const userAccountB = await mintB.createAccount(owner.publicKey);
    await mintB.mintTo(userAccountB, owner, [], tokenBAmount);
    await mintB.approve(
      userAccountB,
      userTransferAuthority.publicKey,
      owner,
      [],
      tokenBAmount,
    );
    // Creating depositor pool token account
    const newAccountPool = await tokenPool.createAccount(owner.publicKey);

    // Depositing into swap
    await program.rpc.depositAllTokenTypes(
      new anchor.BN(POOL_TOKEN_AMOUNT),
      new anchor.BN(tokenAAmount),
      new anchor.BN(tokenBAmount),
      {
        accounts: {
          authority: authority,
          amm: ammAccount.publicKey,
          userTransferAuthorityInfo: userTransferAuthority.publicKey,
          sourceAInfo: userAccountA,
          sourceBInfo: userAccountB,
          tokenA: tokenAccountA,
          tokenB: tokenAccountB,
          poolMint: tokenPool.publicKey,
          destination: newAccountPool,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [userTransferAuthority]
      }
    );

    let info;
    info = await mintA.getAccountInfo(userAccountA);
    assert(info.amount.toNumber() == 0);
    info = await mintB.getAccountInfo(userAccountB);
    assert(info.amount.toNumber() == 0);
    info = await mintA.getAccountInfo(tokenAccountA);
    assert(info.amount.toNumber() == currentSwapTokenA + tokenAAmount);
    currentSwapTokenA += tokenAAmount;
    info = await mintB.getAccountInfo(tokenAccountB);
    assert(info.amount.toNumber() == currentSwapTokenB + tokenBAmount);
    currentSwapTokenB += tokenBAmount;
    info = await tokenPool.getAccountInfo(newAccountPool);
    assert(info.amount.toNumber() == POOL_TOKEN_AMOUNT);
  });

  it("WithdrawAllTokenTypes", async () => {
    const poolMintInfo = await tokenPool.getMintInfo();
    const supply = (poolMintInfo.supply as anchor.BN).toNumber();
    let swapTokenA = await mintA.getAccountInfo(tokenAccountA);
    let swapTokenB = await mintB.getAccountInfo(tokenAccountB);
    let feeAmount = 0;
    if (OWNER_WITHDRAW_FEE_NUMERATOR !== 0) {
      feeAmount = Math.floor(
        (POOL_TOKEN_AMOUNT * OWNER_WITHDRAW_FEE_NUMERATOR) /
          OWNER_WITHDRAW_FEE_DENOMINATOR,
      );
    }
    const poolTokenAmount = POOL_TOKEN_AMOUNT - feeAmount;
    const tokenAAmount = Math.floor(
      ((swapTokenA.amount as anchor.BN).toNumber() * poolTokenAmount) / supply,
    );
    const tokenBAmount = Math.floor(
      ((swapTokenB.amount as anchor.BN).toNumber() * poolTokenAmount) / supply,
    );
  
    // Creating withdraw token A account
    let userAccountA = await mintA.createAccount(owner.publicKey);
    // Creating withdraw token B account
    let userAccountB = await mintB.createAccount(owner.publicKey);
  
    const userTransferAuthority = anchor.web3.Keypair.generate();

    // Approving withdrawal from pool account
    await tokenPool.approve(
      tokenAccountPool,
      userTransferAuthority.publicKey,
      owner,
      [],
      POOL_TOKEN_AMOUNT,
    );
  
    // Withdrawing pool tokens for A and B tokens
    await program.rpc.withdrawAllTokenTypes(
      new anchor.BN(POOL_TOKEN_AMOUNT),
      new anchor.BN(tokenAAmount),
      new anchor.BN(tokenBAmount),
      {
        accounts: {
          amm: ammAccount.publicKey,
          authority: authority,
          userTransferAuthorityInfo: userTransferAuthority.publicKey,
          sourceInfo: tokenAccountPool,
          tokenA: tokenAccountA,
          tokenB: tokenAccountB,
          poolMint: tokenPool.publicKey,
          destTokenAInfo: userAccountA,
          destTokenBInfo: userAccountB,
          feeAccount: feeAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [userTransferAuthority]
      }
    )
  
    swapTokenA = await mintA.getAccountInfo(tokenAccountA);
    swapTokenB = await mintB.getAccountInfo(tokenAccountB);
  
    let info = await tokenPool.getAccountInfo(tokenAccountPool);
    assert(
      (info.amount as anchor.BN).toNumber() == DEFAULT_POOL_TOKEN_AMOUNT - POOL_TOKEN_AMOUNT,
    );
    assert((swapTokenA.amount as anchor.BN).toNumber() == currentSwapTokenA - tokenAAmount);
    currentSwapTokenA -= tokenAAmount;
    assert((swapTokenB.amount as anchor.BN).toNumber() == currentSwapTokenB - tokenBAmount);
    currentSwapTokenB -= tokenBAmount;
    info = await mintA.getAccountInfo(userAccountA);
    assert((info.amount as anchor.BN).toNumber() == tokenAAmount);
    info = await mintB.getAccountInfo(userAccountB);
    assert((info.amount as anchor.BN).toNumber() == tokenBAmount);
    info = await tokenPool.getAccountInfo(feeAccount);
    assert((info.amount as anchor.BN).toNumber() == feeAmount);
    currentFeeAmount = feeAmount;
  });

  it("Swap", async () => {
    // Creating swap token a account
    let userAccountA = await mintA.createAccount(owner.publicKey);
    await mintA.mintTo(userAccountA, owner, [], SWAP_AMOUNT_IN);
    const userTransferAuthority = anchor.web3.Keypair.generate();
    await mintA.approve(
      userAccountA,
      userTransferAuthority.publicKey,
      owner,
      [],
      SWAP_AMOUNT_IN,
    );
    // Creating swap token b account
    let userAccountB = await mintB.createAccount(owner.publicKey);

    let poolAccount = SWAP_PROGRAM_OWNER_FEE_ADDRESS
      ? await tokenPool.createAccount(owner.publicKey)
      : PublicKey.default;

    // Swapping

    await program.rpc.swap(
      new anchor.BN(SWAP_AMOUNT_IN),
      new anchor.BN(SWAP_AMOUNT_OUT),
      {
        accounts: {
          authority: authority,
          amm: ammAccount.publicKey,
          userTransferAuthority: userTransferAuthority.publicKey,
          sourceInfo: userAccountA,
          destinationInfo: userAccountB,
          swapSource: tokenAccountA,
          swapDestination: tokenAccountB,
          poolMint: tokenPool.publicKey,
          feeAccount: feeAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          hostFeeAccount: PublicKey.default,
        },
        signers: [userTransferAuthority]
      }
    );

    let info;
    info = await mintA.getAccountInfo(userAccountA);
    assert(info.amount.toNumber() == 0);

    info = await mintB.getAccountInfo(userAccountB);
    assert(info.amount.toNumber() == SWAP_AMOUNT_OUT);

    info = await mintA.getAccountInfo(tokenAccountA);
    assert(info.amount.toNumber() == currentSwapTokenA + SWAP_AMOUNT_IN);
    currentSwapTokenA += SWAP_AMOUNT_IN;

    info = await mintB.getAccountInfo(tokenAccountB);
    assert(info.amount.toNumber() == currentSwapTokenB - SWAP_AMOUNT_OUT);
    currentSwapTokenB -= SWAP_AMOUNT_OUT;

    info = await tokenPool.getAccountInfo(tokenAccountPool);
    assert(
      info.amount.toNumber() == DEFAULT_POOL_TOKEN_AMOUNT - POOL_TOKEN_AMOUNT,
    );

    info = await tokenPool.getAccountInfo(feeAccount);
    assert(info.amount.toNumber() == currentFeeAmount + OWNER_SWAP_FEE);

    if (poolAccount != PublicKey.default) {
      info = await tokenPool.getAccountInfo(poolAccount);
      assert(info.amount.toNumber() == HOST_SWAP_FEE);
    }
  });

  it("DepositSingleTokenType", async () => {
    const tradingTokensToPoolTokens = (sourceAmount: number, swapSourceAmount: number, poolAmount: number): number => {
      const tradingFee =
        (sourceAmount / 2) * (TRADING_FEE_NUMERATOR / TRADING_FEE_DENOMINATOR);
      const sourceAmountPostFee = sourceAmount - tradingFee;
      const root = Math.sqrt(sourceAmountPostFee / swapSourceAmount + 1);
      return Math.floor(poolAmount * (root - 1));
    }
    
    // Pool token amount to deposit on one side
    const depositAmount = 10000;

    const poolMintInfo = await tokenPool.getMintInfo();
    const supply = (poolMintInfo.supply as anchor.BN).toNumber();
    const swapTokenA = await mintA.getAccountInfo(tokenAccountA);
    const poolTokenAAmount = tradingTokensToPoolTokens(
      depositAmount,
      (swapTokenA.amount as anchor.BN).toNumber(),
      supply,
    );
    const swapTokenB = await mintB.getAccountInfo(tokenAccountB);
    const poolTokenBAmount = tradingTokensToPoolTokens(
      depositAmount,
      (swapTokenB.amount as anchor.BN).toNumber(),
      supply,
    );

    const userTransferAuthority = anchor.web3.Keypair.generate();
    // Creating depositor token a account
    const userAccountA = await mintA.createAccount(owner.publicKey);
    await mintA.mintTo(userAccountA, owner, [], depositAmount);
    await mintA.approve(
      userAccountA,
      userTransferAuthority.publicKey,
      owner,
      [],
      depositAmount,
    );
    // Creating depositor token b account
    const userAccountB = await mintB.createAccount(owner.publicKey);
    await mintB.mintTo(userAccountB, owner, [], depositAmount);
    await mintB.approve(
      userAccountB,
      userTransferAuthority.publicKey,
      owner,
      [],
      depositAmount,
    );
    // Creating depositor pool token account
    const newAccountPool = await tokenPool.createAccount(owner.publicKey);

    // Depositing token A into swap
    await program.rpc.depositSingleTokenType(
      new anchor.BN(depositAmount),
      new anchor.BN(poolTokenAAmount),
      {
        accounts: {
          amm: ammAccount.publicKey,
          authority: authority,
          userTransferAuthorityInfo: userTransferAuthority.publicKey,
          source: userAccountA,
          swapTokenA: tokenAccountA,
          swapTokenB: tokenAccountB,
          poolMint: tokenPool.publicKey,
          destination: newAccountPool,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [userTransferAuthority]
      });

    let info;
    info = await mintA.getAccountInfo(userAccountA);
    assert(info.amount.toNumber() == 0);
    info = await mintA.getAccountInfo(tokenAccountA);
    assert(info.amount.toNumber() == currentSwapTokenA + depositAmount);
    currentSwapTokenA += depositAmount;

    // Depositing token B into swap
    await program.rpc.depositSingleTokenType(
      new anchor.BN(depositAmount),
      new anchor.BN(poolTokenBAmount),
      {
        accounts: {
          amm: ammAccount.publicKey,
          authority: authority,
          userTransferAuthorityInfo: userTransferAuthority.publicKey,
          source: userAccountB,
          swapTokenA: tokenAccountA,
          swapTokenB: tokenAccountB,
          poolMint: tokenPool.publicKey,
          destination: newAccountPool,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [userTransferAuthority]
      });

    info = await mintB.getAccountInfo(userAccountB);
    assert(info.amount.toNumber() == 0);
    info = await mintB.getAccountInfo(tokenAccountB);
    assert(info.amount.toNumber() == currentSwapTokenB + depositAmount);
    currentSwapTokenB += depositAmount;
    info = await tokenPool.getAccountInfo(newAccountPool);
    assert(info.amount.toNumber() >= poolTokenAAmount + poolTokenBAmount);
  });

  it("WithdrawSingleTokenType", async () => {
    const tradingTokensToPoolTokens = (sourceAmount: number, swapSourceAmount: number, poolAmount: number): number => {
      const tradingFee =
        (sourceAmount / 2) * (TRADING_FEE_NUMERATOR / TRADING_FEE_DENOMINATOR);
      const sourceAmountPostFee = sourceAmount - tradingFee;
      const root = Math.sqrt(sourceAmountPostFee / swapSourceAmount + 1);
      return Math.floor(poolAmount * (root - 1));
    }

    // Pool token amount to withdraw on one side
    const withdrawAmount = 50000;
    const roundingAmount = 1.0001; // make math a little easier

    const poolMintInfo = await tokenPool.getMintInfo();
    const supply = (poolMintInfo.supply as anchor.BN).toNumber();

    const swapTokenA = await mintA.getAccountInfo(tokenAccountA);
    const swapTokenAPost = (swapTokenA.amount as anchor.BN).toNumber() - withdrawAmount;
    const poolTokenA = tradingTokensToPoolTokens(
      withdrawAmount,
      swapTokenAPost,
      supply,
    );
    let adjustedPoolTokenA = poolTokenA * roundingAmount;
    if (OWNER_WITHDRAW_FEE_NUMERATOR !== 0) {
      adjustedPoolTokenA *=
        1 + OWNER_WITHDRAW_FEE_NUMERATOR / OWNER_WITHDRAW_FEE_DENOMINATOR;
    }

    const swapTokenB = await mintB.getAccountInfo(tokenAccountB);
    const swapTokenBPost = (swapTokenB.amount as anchor.BN).toNumber() - withdrawAmount;
    const poolTokenB = tradingTokensToPoolTokens(
      withdrawAmount,
      swapTokenBPost,
      supply,
    );
    let adjustedPoolTokenB = poolTokenB * roundingAmount;
    if (OWNER_WITHDRAW_FEE_NUMERATOR !== 0) {
      adjustedPoolTokenB *=
        1 + OWNER_WITHDRAW_FEE_NUMERATOR / OWNER_WITHDRAW_FEE_DENOMINATOR;
    }

    const userTransferAuthority = anchor.web3.Keypair.generate();
    // Creating withdraw token a account
    const userAccountA = await mintA.createAccount(owner.publicKey);
    // Creating withdraw token b account
    const userAccountB = await mintB.createAccount(owner.publicKey);
    // Creating withdraw pool token account
    const poolAccount = await tokenPool.getAccountInfo(tokenAccountPool);
    const poolTokenAmount = (poolAccount.amount as anchor.BN).toNumber();
    await tokenPool.approve(
      tokenAccountPool,
      userTransferAuthority.publicKey,
      owner,
      [],
      adjustedPoolTokenA + adjustedPoolTokenB,
    );

    // Withdrawing token A only
    await program.rpc.withdrawSingleTokenType(
      new anchor.BN(withdrawAmount),
      new anchor.BN(adjustedPoolTokenA),
      {
        accounts: {
          amm: ammAccount.publicKey,
          authority: authority,
          userTransferAuthorityInfo: userTransferAuthority.publicKey,
          source: tokenAccountPool,
          swapTokenA: tokenAccountA,
          swapTokenB: tokenAccountB,
          poolMint: tokenPool.publicKey,
          destination: userAccountA,
          feeAccount: feeAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [userTransferAuthority]
      });


    let info;
    info = await mintA.getAccountInfo(userAccountA);
    assert(info.amount.toNumber() == withdrawAmount);
    info = await mintA.getAccountInfo(tokenAccountA);
    assert(info.amount.toNumber() == currentSwapTokenA - withdrawAmount);
    currentSwapTokenA += withdrawAmount;
    info = await tokenPool.getAccountInfo(tokenAccountPool);
    assert(info.amount.toNumber() >= poolTokenAmount - adjustedPoolTokenA);

    // Withdrawing token B only
    await program.rpc.withdrawSingleTokenType(
      new anchor.BN(withdrawAmount),
      new anchor.BN(adjustedPoolTokenB),
      {
        accounts: {
          amm: ammAccount.publicKey,
          authority: authority,
          userTransferAuthorityInfo: userTransferAuthority.publicKey,
          source: tokenAccountPool,
          swapTokenA: tokenAccountA,
          swapTokenB: tokenAccountB,
          poolMint: tokenPool.publicKey,
          destination: userAccountB,
          feeAccount: feeAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [userTransferAuthority]
      });

    info = await mintB.getAccountInfo(userAccountB);
    assert(info.amount.toNumber() == withdrawAmount);
    info = await mintB.getAccountInfo(tokenAccountB);
    assert(info.amount.toNumber() == currentSwapTokenB - withdrawAmount);
    currentSwapTokenB += withdrawAmount;
    info = await tokenPool.getAccountInfo(tokenAccountPool);
    assert(
      info.amount.toNumber() >=
        poolTokenAmount - adjustedPoolTokenA - adjustedPoolTokenB,
    );
  })
});
