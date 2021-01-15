const assert = require('assert');
//const anchor = require('@project-serum/anchor');
const anchor = require('/home/armaniferrante/Documents/code/src/github.com/project-serum/anchor/ts');

describe('basic-2', () => {

  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  // Author for the tests.
  const author = new anchor.web3.Account();

  // Program for the tests.
  const program = anchor.workspace.Basic2;

  it('Creates an author', async () => {
    await program.rpc.createAuthor(provider.wallet.publicKey, 'Ghost', {
      accounts: {
        author: author.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [author],
      instructions: [
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: author.publicKey,
          space: 8+1000,
          lamports: await provider.connection.getMinimumBalanceForRentExemption(8+1000),
          programId: program.programId,
        }),
      ],
    });

    let authorAccount = await program.account.author(author.publicKey);

    assert.ok(authorAccount.authority.equals(provider.wallet.publicKey));
    assert.ok(authorAccount.name === 'Ghost');
  });

  it('Updates an author', async () => {
    await program.rpc.updateAuthor('Updated author', {
      accounts: {
        author: author.publicKey,
        authority: provider.wallet.publicKey,
      },
    });

    authorAccount = await program.account.author(author.publicKey);

    assert.ok(authorAccount.authority.equals(provider.wallet.publicKey));
    assert.ok(authorAccount.name === 'Updated author');
  });

  it('Creates a book', async () => {

  });
});
