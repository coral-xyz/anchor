import { web3, utils, Provider, Program, SplToken, BN } from "../src";
import NodeWallet from "../src/nodewallet";
import { program } from "../src/spl/token";

export const DEFAULT_RPC_ENDPOINT = "https://api.testnet.solana.com";

describe("SPL", () => {
  // Internal setup
  const _wallet = new NodeWallet(new web3.Keypair());
  const _connection = new web3.Connection(DEFAULT_RPC_ENDPOINT, "confirmed");
  const _provider = new Provider(_connection, _wallet, {
    skipPreflight: true,
    commitment: "confirmed",
  });
  // Public instance
  let splProgram: Program<SplToken>,
    token: web3.Keypair,
    decimals: number,
    associatedTokenAccount: web3.PublicKey;

  beforeAll(async () => {
    // Setup instances
    splProgram = program(_provider);
    token = new web3.Keypair();
    decimals = 6;
    associatedTokenAccount = await utils.token.associatedAddress({
      mint: token.publicKey,
      owner: splProgram.provider.wallet.publicKey,
    });
    // Prepare lamports for main account
    await splProgram.provider.connection.requestAirdrop(
      splProgram.provider.wallet.publicKey,
      1_000_000_000
    );
  });

  it("should initialize a mint", async () => {
    const ix = await splProgram.account.mint.createInstruction(token);
    const tx = new web3.Transaction().add(ix);
    await splProgram.provider.send(tx, [token]);
    await splProgram.rpc.initializeMint(
      decimals,
      splProgram.provider.wallet.publicKey,
      splProgram.provider.wallet.publicKey,
      {
        accounts: {
          mint: token.publicKey,
          rent: web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [],
      }
    );
  });

  it("should initialize an associated account", async () => {
    const ix = new web3.TransactionInstruction({
      keys: [
        {
          pubkey: splProgram.provider.wallet.publicKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: associatedTokenAccount,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: splProgram.provider.wallet.publicKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: token.publicKey,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: splProgram.programId,
          isSigner: false,
          isWritable: false,
        },
        {
          pubkey: web3.SYSVAR_RENT_PUBKEY,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: utils.token.ASSOCIATED_PROGRAM_ID,
      data: Buffer.from([]),
    });
    const tx = new web3.Transaction().add(ix);
    await splProgram.provider.send(tx);
  });

  it("should mint 1 token to the token account", async () => {
    const amount = new BN(10 ** decimals);
    await splProgram.rpc.mintTo(amount, {
      accounts: {
        mint: token.publicKey,
        to: associatedTokenAccount,
        authority: splProgram.provider.wallet.publicKey,
      },
    });
  });

  it("should transfer 0.5 token", async () => {
    // Create a dummy receiver
    const dstTokenAccount = new web3.Keypair();
    const ix = await splProgram.account.token.createInstruction(
      dstTokenAccount
    );
    const tx = new web3.Transaction().add(ix);
    await splProgram.provider.send(tx, [dstTokenAccount]); // Rent acc
    await splProgram.rpc.initializeAccount({
      accounts: {
        account: dstTokenAccount.publicKey,
        mint: token.publicKey,
        authority: splProgram.provider.wallet.publicKey,
        rent: web3.SYSVAR_RENT_PUBKEY,
      },
    });
    // Transfer
    await splProgram.rpc.transfer(new BN(0.5 * 10 ** decimals), {
      accounts: {
        source: associatedTokenAccount,
        destination: dstTokenAccount.publicKey,
        authority: splProgram.provider.wallet.publicKey,
      },
    });
  });

  it("should read mint data", async () => {
    const { supply } = await splProgram.account.mint.fetch(token.publicKey);
    console.log("supply", supply.toNumber());
    expect(supply.toNumber()).toBe(10 ** decimals);
  });

  it("should read account data", async () => {
    const { amount } = await splProgram.account.token.fetch(
      associatedTokenAccount
    );
    console.log("amount", amount.toNumber());
    expect(amount.toNumber()).toBe(0.5 * 10 ** decimals);
  });
});
