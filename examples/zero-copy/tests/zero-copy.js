//const anchor = require('@project-serum/anchor');
const anchor = require("/home/armaniferrante/Documents/code/src/github.com/project-serum/anchor/ts");
const assert = require("assert");

describe("zero-copy", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.ZeroCopy;

  const foo = new anchor.web3.Account();

  it("Is creates a zero copy account", async () => {
    await program.rpc.createFoo({
      accounts: {
        foo: foo.publicKey,
        authority: program.provider.wallet.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      instructions: [await program.account.foo.createInstruction(foo)],
      signers: [foo],
    });
    const account = await program.account.foo(foo.publicKey);
    assert.ok(JSON.stringify(account.authority), [
      ...program.provider.wallet.publicKey.toBuffer(),
    ]);
    assert.ok(account.data.toNumber() === 0);
    assert.ok(account.moreData.toNumber() === 0);
    assert.ok(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...new anchor.web3.PublicKey().toBuffer()])
    );
  });

  it("Updates a zero copy account", async () => {
    await program.rpc.updateFoo(new anchor.BN(1234), new anchor.BN(54), {
      accounts: {
        foo: foo.publicKey,
        authority: program.provider.wallet.publicKey,
      },
    });

    const account = await program.account.foo(foo.publicKey);

    assert.ok(JSON.stringify(account.authority), [
      ...program.provider.wallet.publicKey.toBuffer(),
    ]);
    assert.ok(account.data.toNumber() === 1234);
    assert.ok(account.moreData.toNumber() === 54);
    assert.ok(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...new anchor.web3.PublicKey().toBuffer()])
    );
  });
});
