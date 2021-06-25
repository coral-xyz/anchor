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
    
    let [pdaSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    usdcMint = await utils.createMint(program.provider, provider.wallet.publicKey, decimals);
    // Give the provider account a bunch of USDC to play around with
    fromUsdc = await utils.createTokenAccount(provider, usdcMint, provider.wallet.publicKey);
    await utils.mintToAccount(provider, usdcMint, fromUsdc, new anchor.BN(1000000000000000), provider.wallet.publicKey);
    toUsdc = await utils.createTokenAccount(provider, usdcMint, pdaSigner);
    await utils.mintToAccount(provider, usdcMint, toUsdc, new anchor.BN(1000000000000000), provider.wallet.publicKey);


    // Transfer over a dollar at time
    const oneDollar = new anchor.BN(1000000);

    for (let i = 27; i < 28; i++) {
      let numTransfers = new anchor.BN(i);
      console.log("Num transfers: ", numTransfers);
      let tx = await program.rpc.transferPlease(numTransfers, {
        accounts: {
          fromUsdc,
          toUsdc,
          authority: provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID
        }
      });
    };

    await program.rpc.initSigner(nonce, {
      accounts: {
        pdaSigner,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      }
    });
    
    let pdaSignerData = await program.account.pdaSigner.fetch(
      pdaSigner
    );
    console.log(pdaSignerData);


    for (let i = 27; i < 28; i++) {
      let numTransfers = new anchor.BN(i);
      console.log("Num transfers: ", numTransfers);
      await program.rpc.signedTransfer(numTransfers, {
        accounts: {
          pdaSigner,
          fromUsdc,
          toUsdc,
          authority: provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID
        }
      });
    };



    // console.log("Your transaction signature", tx);
  });
});
