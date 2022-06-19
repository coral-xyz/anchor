import * as anchor from "@project-serum/anchor";
import { Serum } from "@project-serum/anchor";
import {
  Account,
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {} from "@solana/spl-token";
import * as assert from "assert";
import {
  createMint,
  createMintAndVault,
  createMintInstructions,
  createTokenAccountInstrs,
  NodeWallet,
  Provider,
} from "@project-serum/common";

const DEX_STATE_LEN = 280; // dex-v4's Dex State
const MARKET_STATE_LEN = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8; // Central AAOB Market State
const EVENT_QUEUE_HEADER_LEN = 33;
const REGISTER_SIZE = 41 + 1; // ORDER_SUMMARY_SIZE + 1
const ORDERBOOK_LEN = 1_000_00;

const QUOTE_DECIMALS = 6;
const BASE_DECIMALS = 6;

const METAPLEX_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const computeSlotSize = (callbackInfoLength: number) => {
  return 1 + 33 + 2 * callbackInfoLength;
};

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
  // const connection = new Connection("http://localhost:8899", "confirmed");
  // const wallet = NodeWallet.local();
  // const provider = new Provider(
  //   connection,
  //   wallet,
  //   anchor.AnchorProvider.defaultOptions()
  // );
  const anchorProvider = anchor.AnchorProvider.env();
  anchor.setProvider(anchorProvider);

  const program = Serum.dex(anchorProvider);

  const { wallet, connection } = anchorProvider;

  const provider = new Provider(
    connection,
    wallet,
    anchor.AnchorProvider.defaultOptions()
  );

  // const { connection, wallet } = provider;

  const bob = Keypair.generate();

  const createDexAccounts = async () => {
    const baseMintKP = Keypair.generate();
    const quoteMintKP = Keypair.generate();
    const dexMarketKP = Keypair.generate();
    const baseVaultKP = Keypair.generate();
    const quoteVaultKP = Keypair.generate();
    const marketAdminKP = Keypair.generate();

    const [marketSigner, marketSignerNonce] =
      await PublicKey.findProgramAddress(
        [dexMarketKP.publicKey.toBuffer()],
        program.programId
      );

    const baseMintIxs = await createMintInstructions(
      provider,
      provider.wallet.publicKey,
      baseMintKP.publicKey,
      QUOTE_DECIMALS
    );
    const quoteMintIxs = await createMintInstructions(
      provider,
      provider.wallet.publicKey,
      quoteMintKP.publicKey,
      BASE_DECIMALS
    );
    const baseVaultIxs = await createTokenAccountInstrs(
      provider,
      baseVaultKP.publicKey,
      baseMintKP.publicKey,
      marketSigner
    );
    const quoteVaultIxs = await createTokenAccountInstrs(
      provider,
      quoteVaultKP.publicKey,
      quoteMintKP.publicKey,
      marketSigner
    );

    const dexMarketIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: dexMarketKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        DEX_STATE_LEN
      ),
      space: DEX_STATE_LEN,
      programId: program.programId,
    });

    let tx = new Transaction();
    tx.add(
      dexMarketIx,
      ...baseMintIxs,
      ...quoteMintIxs,
      ...quoteVaultIxs,
      ...baseVaultIxs
    );
    const signers = [
      dexMarketKP,
      quoteMintKP,
      baseMintKP,
      baseVaultKP,
      quoteVaultKP,
    ];
    await provider.send(
      tx,
      signers.map((kp) => new Account(kp.secretKey))
    );

    return {
      dexMarket: dexMarketKP,
      baseMint: baseMintKP,
      quoteMint: quoteMintKP,
      baseVault: baseVaultKP,
      quoteVault: quoteVaultKP,
      marketSigner,
      marketSignerNonce,
      marketAdmin: marketAdminKP,
    };
  };

  const createAaobAccounts = async () => {
    const marketKP = Keypair.generate();
    const eventQKP = Keypair.generate();
    const bidsKP = Keypair.generate();
    const asksKP = Keypair.generate();

    const marketIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: marketKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        MARKET_STATE_LEN
      ),
      space: MARKET_STATE_LEN,
      programId: program.programId,
    });

    const eventQSize =
      EVENT_QUEUE_HEADER_LEN + REGISTER_SIZE + 10 * computeSlotSize(33);
    const eventQIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: eventQKP.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(eventQSize),
      space: eventQSize,
      programId: program.programId,
    });

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

    const signers = [marketKP, eventQKP, bidsKP, asksKP];
    await provider.send(
      tx,
      signers.map((kp) => new Account(kp.secretKey))
    );

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

    const metadataAccount = await getMPLMetadataAccount(
      dexAccounts.baseMint.publicKey
    );

    const txSig = await program.methods
      .createMarket(
        new anchor.BN(dexAccounts.marketSignerNonce),
        new anchor.BN(10),
        new anchor.BN(1),
        new anchor.BN(0),
        new anchor.BN(1),
        new anchor.BN(1)
      )
      .accounts({
        market: dexAccounts.dexMarket.publicKey,
        orderbook: aaobAccounts.market.publicKey,
        baseVault: dexAccounts.baseVault.publicKey,
        quoteVault: dexAccounts.quoteVault.publicKey,
        eventQueue: aaobAccounts.eventQ.publicKey,
        bids: aaobAccounts.bids.publicKey,
        asks: aaobAccounts.asks.publicKey,
        marketAdmin: dexAccounts.marketAdmin.publicKey,
        tokenMetadata: metadataAccount,
      })
      .rpc();

    console.log(txSig);
  });
});
