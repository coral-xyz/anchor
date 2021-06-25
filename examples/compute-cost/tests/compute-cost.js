const anchor = require('@project-serum/anchor');
const utils = require("./utils");
const TOKEN_PROGRAM_ID = require("@solana/spl-token").TOKEN_PROGRAM_ID;


describe('compute-cost', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  // These are all of the variables we assume exist in the world already and
  // are available to the client.
  let usdcMint = null;
  const decimals = 6;

  it('Is initialized!', async () => {
    // Add your test here.
    const program = anchor.workspace.ComputeCost;
    let provider = program.provider;

    usdcMint = await utils.createMint(program.provider, provider.wallet.publicKey, decimals);
    // Give the provider account a bunch of USDC to play around with
    fromUsdc = await utils.createTokenAccount(provider, usdcMint, provider.wallet.publicKey);
    await utils.mintToAccount(provider, usdcMint, fromUsdc, new anchor.BN(1000000000000000), provider.wallet.publicKey);
    toUsdc = await utils.createTokenAccount(provider, usdcMint, provider.wallet.publicKey);

    // Transfer over a dollar at time
    const oneDollar = new anchor.BN(1000000);
    const numTransfers = new anchor.BN(20);
    const tx = await program.rpc.transferPlease(numTransfers, {
      accounts: {
        fromUsdc,
        toUsdc,
        authority: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID
      }
    });
    console.log("Your transaction signature", tx);
  });
});
