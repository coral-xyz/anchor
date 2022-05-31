import * as anchor from "@project-serum/anchor";
import { Serum } from "@project-serum/anchor";
import { createMint, NodeWallet, Provider } from "@project-serum/common";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import * as assert from "assert";

const DEX_STATE_LEN = 264; // dex-v4's Dex State
const MARKET_STATE_LEN = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8; // Central AAOB Market State
const EVENT_QUEUE_HEADER_LEN = 33;
const REGISTER_SIZE = 41 + 1; // ORDER_SUMMARY_SIZE + 1
const ORDERBOOK_LEN = 1_000_00;

const METAPLEX_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

async function getAssociatedTokenAddress(
  mint: PublicKey,
  owner: PublicKey,
  allowOwnerOffCurve = false,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID
): Promise<PublicKey> {
  if (!allowOwnerOffCurve && !PublicKey.isOnCurve(owner.toBuffer()))
    throw new Error("Owner is off curve");

  const [address] = await PublicKey.findProgramAddress(
    [owner.toBuffer(), programId.toBuffer(), mint.toBuffer()],
    associatedTokenProgramId
  );

  return address;
}

function createAssociatedTokenAccountInstruction(
  payer: PublicKey,
  associatedToken: PublicKey,
  owner: PublicKey,
  mint: PublicKey,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID
): TransactionInstruction {
  const keys = [
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: associatedToken, isSigner: false, isWritable: true },
    { pubkey: owner, isSigner: false, isWritable: false },
    { pubkey: mint, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: programId, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
  ];

  return new TransactionInstruction({
    keys,
    programId: associatedTokenProgramId,
    data: Buffer.alloc(0),
  });
}

async function getMPLMetadataAccount(mint: PublicKey) {
  const [metadataAccount] = await PublicKey.findProgramAddress(
    [
      Buffer.from("metadata"),
      METAPLEX_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    METAPLEX_METADATA_PROGRAM_ID
  );
  return metadataAccount;
}

describe("serum-coder", () => {
  const connection = new Connection(clusterApiUrl("devnet"), "confirmed");
  const wallet = NodeWallet.local();

  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    anchor.AnchorProvider.defaultOptions()
  );
  anchor.setProvider(provider);

  const program = Serum.dex(provider);

  // console.log(wallet.publicKey.toString());

  const bobKeypair = Keypair.generate();

  const computeSlotSize = (callbackInfoLength: number) => {
    return 1 + 33 + 2 * callbackInfoLength;
  };

  const createDexAccounts = async () => {
    const baseMintAuthKP = Keypair.generate();
    const quoteMintAuthKP = Keypair.generate();

    const baseMint = await createMint(
      new Provider(connection, wallet, { commitment: "confirmed" }),
      baseMintAuthKP.publicKey,
      6
    );
    const quoteMint = await createMint(
      new Provider(connection, wallet, { commitment: "confirmed" }),
      quoteMintAuthKP.publicKey,
      6
    );

    const dexMarketKP = Keypair.generate();
    const dexMarketIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: dexMarketKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        DEX_STATE_LEN
      ),
      space: DEX_STATE_LEN,
      programId: program.programId,
    });

    const [marketSigner, marketSignerNonce] =
      await PublicKey.findProgramAddress(
        [dexMarketKP.publicKey.toBuffer()],
        program.programId
      );

    const baseVault = await getAssociatedTokenAddress(
      baseMint,
      marketSigner,
      true
    );
    const quoteVault = await getAssociatedTokenAddress(
      quoteMint,
      marketSigner,
      true
    );

    const baseVaultIx = createAssociatedTokenAccountInstruction(
      wallet.publicKey,
      baseVault,
      marketSigner,
      baseMint
    );
    const quoteVaultIx = createAssociatedTokenAccountInstruction(
      wallet.publicKey,
      quoteVault,
      marketSigner,
      quoteMint
    );

    const tx = new Transaction();
    tx.add(dexMarketIx, quoteVaultIx, baseVaultIx);

    const sig = await connection.sendTransaction(tx, [
      wallet.payer,
      dexMarketKP,
    ]);
    await connection.confirmTransaction(sig);

    const marketAdmin = Keypair.generate();

    return {
      dexMarket: dexMarketKP,
      baseMintAuth: baseMintAuthKP,
      quoteMintAuth: quoteMintAuthKP,
      baseMint,
      quoteMint,
      baseVault,
      quoteVault,
      marketSigner,
      marketSignerNonce,
      marketAdmin,
    };
  };

  const createAaobAccounts = async () => {
    const marketKP = Keypair.generate();
    const marketIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: marketKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        MARKET_STATE_LEN
      ),
      space: MARKET_STATE_LEN,
      programId: program.programId,
    });

    const eventQKP = Keypair.generate();
    const eventQSize =
      EVENT_QUEUE_HEADER_LEN + REGISTER_SIZE + 10 * computeSlotSize(33);
    const eventQIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: eventQKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(eventQSize),
      space: eventQSize,
      programId: program.programId,
    });

    const bidsKP = Keypair.generate();
    const asksKP = Keypair.generate();

    const bidsIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: bidsKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        ORDERBOOK_LEN
      ),
      space: ORDERBOOK_LEN,
      programId: program.programId,
    });
    const asksIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: asksKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        ORDERBOOK_LEN
      ),
      space: ORDERBOOK_LEN,
      programId: program.programId,
    });

    const tx = new Transaction();
    tx.add(marketIx, eventQIx, bidsIx, asksIx);

    const sig = await connection.sendTransaction(tx, [
      wallet.payer,
      marketKP,
      eventQKP,
      bidsKP,
      asksKP,
    ]);
    await connection.confirmTransaction(sig);

    return {
      market: marketKP,
      eventQ: eventQKP,
      bids: bidsKP,
      asks: asksKP,
    };
  };

  it("can create market", async () => {
    const dexAccounts = await createDexAccounts();
    const aaobAccounts = await createAaobAccounts();

    const metadataAccount = await getMPLMetadataAccount(dexAccounts.baseMint);

    const sig = await program.methods
      .createMarket(
        new anchor.BN(dexAccounts.marketSignerNonce),
        new anchor.BN(10),
        new anchor.BN(1),
        new anchor.BN(0)
      )
      .accounts({
        market: dexAccounts.dexMarket.publicKey,
        orderbook: aaobAccounts.market.publicKey,
        baseVault: dexAccounts.baseVault,
        quoteVault: dexAccounts.quoteVault,
        eventQueue: aaobAccounts.eventQ.publicKey,
        bids: aaobAccounts.bids.publicKey,
        asks: aaobAccounts.asks.publicKey,
        marketAdmin: dexAccounts.marketAdmin.publicKey,
        tokenMetadata: metadataAccount,
      })
      .rpc();

    assert.notEqual(sig, null);
  });
});
