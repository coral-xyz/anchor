const anchor = require('@project-serum/anchor');
const splToken = require('@solana/spl-token');
const assert = require('assert');

describe('system_accounts', () => {
  anchor.setProvider(anchor.Provider.local());
  const program = anchor.workspace.SystemAccounts;
  const authority = program.provider.wallet.payer;
  const wallet = anchor.web3.Keypair.generate();

  it('Is initialized!', async () => {
    const tx = await program.rpc.initialize({
      accounts: {
        authority: authority.publicKey,
        wallet: wallet.publicKey
      },
      signers: [authority]
    });

    console.log("Your transaction signature", tx);
  });

  it('Emits an AccountNotSystemOwned error', async () => {
    const mint = await splToken.Token.createMint(
      connection,
      wallet,
      wallet.publicKey,
      null,
      9,
      splToken.TOKEN_PROGRAM_ID,
    );

    const tokenAccount = await mint.getOrCreateAssociatedAccountInfo(
      wallet.publicKey
    );

    await mint.mintTo(
      tokenAccount.address,
      wallet.publicKey,
      [],
      1000000000,
    );

    try {
      await program.rpc.initialize({
        accounts: {
          authority: authority.publicKey,
          wallet: tokenAccount.address
        }
      })
      assert.ok(false);
    } catch (err) {
      const errMsg = 'The given account is not owned by the system program';
      assert.equal(err.toString(), errMsg);
      assert.equal(err.msg, errMsg);
      assert.equal(err.code, 171);
    }
  });
});
