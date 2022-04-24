const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { SystemProgram } = anchor.web3;

describe("basic-2", () => {
  const provider = anchor.AnchorProvider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  // Counter for the tests.
  const counter = anchor.web3.Keypair.generate();

  // Program for the tests.
  const program = anchor.workspace.Basic2;

  // The accounts to pass in the rpc
  const accounts = {
    counter: counter.publicKey,
    user: provider.wallet.publicKey,
    systemProgram: SystemProgram.programId,
  };

  // The signers to pass in the rpc
  const signers = [counter];

  it("Creates a counter", async () => {
    await program.methods.create(provider.wallet.publicKey).accounts(accounts).signers(signers).rpc();

    let counterAccount = await program.account.counter.fetch(counter.publicKey);

    assert.ok(counterAccount.authority.equals(provider.wallet.publicKey));
    assert.ok(counterAccount.count.toNumber() === 0);
  });

  it("Updates a counter", async () => {
    // The accounts to pass in the rpc
    const accounts = {
      counter: counter.publicKey,
      authority: provider.wallet.publicKey,
    };

    await program.methods.increment().accounts(accounts).rpc();

    const counterAccount = await program.account.counter.fetch(
      counter.publicKey
    );

    assert.ok(counterAccount.authority.equals(provider.wallet.publicKey));
    assert.ok(counterAccount.count.toNumber() == 1);
  });
});
