const anchor = require('@project-serum/anchor');

describe('token', () => {

  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  it('Is initialized!', async () => {
    const program = anchor.workspace.TokenProxy;

    console.log(program);
  });
});
