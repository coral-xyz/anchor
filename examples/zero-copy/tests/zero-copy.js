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
    assert.ok(account.secondData.toNumber() === 0);
    assert.ok(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
    );
  });

  it("Updates a zero copy account field", async () => {
    await program.rpc.updateFoo(new anchor.BN(1234), {
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
    assert.ok(account.secondData.toNumber() === 0);
    assert.ok(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
    );
  });

  it("Updates a a second zero copy account field", async () => {
    await program.rpc.updateFooSecond(new anchor.BN(55), {
      accounts: {
        foo: foo.publicKey,
        secondAuthority: program.provider.wallet.publicKey,
      },
    });

    const account = await program.account.foo(foo.publicKey);

    assert.ok(JSON.stringify(account.authority), [
      ...program.provider.wallet.publicKey.toBuffer(),
    ]);
    assert.ok(account.data.toNumber() === 1234);
    assert.ok(account.secondData.toNumber() === 55);
    assert.ok(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
    );
  });

  it("Creates an associated zero copy account", async () => {
    await program.rpc.createBar({
      accounts: {
        bar: await program.account.bar.associatedAddress(
          program.provider.wallet.publicKey,
          foo.publicKey
        ),
        authority: program.provider.wallet.publicKey,
        foo: foo.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });

    const bar = await program.account.bar.associated(
      program.provider.wallet.publicKey,
      foo.publicKey
    );
    assert.ok(bar.authority.equals(program.provider.wallet.publicKey));
    assert.ok(bar.data.toNumber() === 0);
  });

  it("Updates an associated zero copy account", async () => {
    await program.rpc.updateBar(new anchor.BN(99), {
      accounts: {
        bar: await program.account.bar.associatedAddress(
          program.provider.wallet.publicKey,
          foo.publicKey
        ),
        authority: program.provider.wallet.publicKey,
      },
    });
    const bar = await program.account.bar.associated(
      program.provider.wallet.publicKey,
      foo.publicKey
    );
    assert.ok(bar.authority.equals(program.provider.wallet.publicKey));
    assert.ok(bar.data.toNumber() === 99);
  });
});
