const anchor = require("@project-serum/anchor");
const { assert } = require("chai");
const nativeAssert = require("assert");
const PublicKey = anchor.web3.PublicKey;
const BN = anchor.BN;

describe("zero-copy", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ZeroCopy;
  const programCpi = anchor.workspace.ZeroCpi;

  const foo = anchor.web3.Keypair.generate();
  it("Is creates a zero copy account", async () => {
    await program.methods
      .createFoo()
      .accounts({
        foo: foo.publicKey,
        authority: program.provider.wallet.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([foo])
      .preInstructions([await program.account.foo.createInstruction(foo)])
      .rpc();
    const account = await program.account.foo.fetch(foo.publicKey);
    assert.strictEqual(
      JSON.stringify(account.authority.toBuffer()),
      JSON.stringify(program.provider.wallet.publicKey.toBuffer())
    );
    assert.strictEqual(account.data.toNumber(), 0);
    assert.strictEqual(account.secondData.toNumber(), 0);
    assert.strictEqual(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
    );
  });

  it("Updates a zero copy account field", async () => {
    await program.methods
      .updateFoo(new BN(1234))
      .accounts({
        foo: foo.publicKey,
        authority: program.provider.wallet.publicKey,
      })
      .rpc();

    const account = await program.account.foo.fetch(foo.publicKey);

    assert.strictEqual(
      JSON.stringify(account.authority.toBuffer()),
      JSON.stringify(program.provider.wallet.publicKey.toBuffer())
    );
    assert.strictEqual(account.data.toNumber(), 1234);
    assert.strictEqual(account.secondData.toNumber(), 0);
    assert.strictEqual(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
    );
  });

  it("Updates a a second zero copy account field", async () => {
    await program.methods
      .updateFooSecond(new BN(55))
      .accounts({
        foo: foo.publicKey,
        secondAuthority: program.provider.wallet.publicKey,
      })
      .rpc();

    const account = await program.account.foo.fetch(foo.publicKey);

    assert.strictEqual(
      JSON.stringify(account.authority.toBuffer()),
      JSON.stringify(program.provider.wallet.publicKey.toBuffer())
    );
    assert.strictEqual(account.data.toNumber(), 1234);
    assert.strictEqual(account.secondData.toNumber(), 55);
    assert.strictEqual(
      JSON.stringify(account.secondAuthority),
      JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
    );
  });

  it("Creates an associated zero copy account", async () => {
    await program.methods
      .createBar()
      .accounts({
        bar: (
          await PublicKey.findProgramAddress(
            [
              program.provider.wallet.publicKey.toBuffer(),
              foo.publicKey.toBuffer(),
            ],
            program.programId
          )
        )[0],
        authority: program.provider.wallet.publicKey,
        foo: foo.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const bar = (
      await PublicKey.findProgramAddress(
        [
          program.provider.wallet.publicKey.toBuffer(),
          foo.publicKey.toBuffer(),
        ],
        program.programId
      )
    )[0];
    const barAccount = await program.account.bar.fetch(bar);
    assert.isTrue(
      barAccount.authority.equals(program.provider.wallet.publicKey)
    );
    assert.strictEqual(barAccount.data.toNumber(), 0);
  });

  it("Updates an associated zero copy account", async () => {
    const bar = (
      await PublicKey.findProgramAddress(
        [
          program.provider.wallet.publicKey.toBuffer(),
          foo.publicKey.toBuffer(),
        ],
        program.programId
      )
    )[0];
    await program.methods
      .updateBar(new BN(99))
      .accounts({
        bar,
        authority: program.provider.wallet.publicKey,
        foo: foo.publicKey,
      })
      .rpc();
    const barAccount = await program.account.bar.fetch(bar);
    assert.isTrue(
      barAccount.authority.equals(program.provider.wallet.publicKey)
    );
    assert.strictEqual(barAccount.data.toNumber(), 99);
    // Check zero_copy CPI
    await programCpi.methods
      .checkCpi(new BN(1337))
      .accounts({
        bar,
        authority: program.provider.wallet.publicKey,
        foo: foo.publicKey,
        zeroCopyProgram: program.programId,
      })
      .rpc();
    const barAccountAfterCpi = await program.account.bar.fetch(bar);
    assert.isTrue(
      barAccountAfterCpi.authority.equals(program.provider.wallet.publicKey)
    );
    assert.strictEqual(barAccountAfterCpi.data.toNumber(), 1337);
  });

  const eventQ = anchor.web3.Keypair.generate();
  const size = 1000000 + 8; // Account size in bytes.

  it("Creates a large event queue", async () => {
    await program.methods
      .createLargeAccount()
      .accounts({
        eventQ: eventQ.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([eventQ])
      .preInstructions([
        await program.account.eventQ.createInstruction(eventQ, size),
      ])
      .rpc();
    const account = await program.account.eventQ.fetch(eventQ.publicKey);
    assert.strictEqual(account.events.length, 25000);
    account.events.forEach((event) => {
      assert.isTrue(event.from.equals(PublicKey.default));
      assert.strictEqual(event.data.toNumber(), 0);
    });
  });

  it("Updates a large event queue", async () => {
    // Set index 0.
    await program.methods
      .updateLargeAccount(0, new BN(48))
      .accounts({
        eventQ: eventQ.publicKey,
        from: program.provider.wallet.publicKey,
      })
      .rpc();
    // Verify update.
    let account = await program.account.eventQ.fetch(eventQ.publicKey);
    assert.strictEqual(account.events.length, 25000);
    account.events.forEach((event, idx) => {
      if (idx === 0) {
        assert.isTrue(event.from.equals(program.provider.wallet.publicKey));
        assert.strictEqual(event.data.toNumber(), 48);
      } else {
        assert.isTrue(event.from.equals(PublicKey.default));
        assert.strictEqual(event.data.toNumber(), 0);
      }
    });

    // Set index 11111.
    await program.methods
      .updateLargeAccount(11111, new BN(1234))
      .accounts({
        eventQ: eventQ.publicKey,
        from: program.provider.wallet.publicKey,
      })
      .rpc();
    // Verify update.
    account = await program.account.eventQ.fetch(eventQ.publicKey);
    assert.strictEqual(account.events.length, 25000);
    account.events.forEach((event, idx) => {
      if (idx === 0) {
        assert.isTrue(event.from.equals(program.provider.wallet.publicKey));
        assert.strictEqual(event.data.toNumber(), 48);
      } else if (idx === 11111) {
        assert.isTrue(event.from.equals(program.provider.wallet.publicKey));
        assert.strictEqual(event.data.toNumber(), 1234);
      } else {
        assert.isTrue(event.from.equals(PublicKey.default));
        assert.strictEqual(event.data.toNumber(), 0);
      }
    });

    // Set last index.
    await program.methods
      .updateLargeAccount(24999, new BN(99))
      .accounts({
        eventQ: eventQ.publicKey,
        from: program.provider.wallet.publicKey,
      })
      .rpc();
    // Verify update.
    account = await program.account.eventQ.fetch(eventQ.publicKey);
    assert.strictEqual(account.events.length, 25000);
    account.events.forEach((event, idx) => {
      if (idx === 0) {
        assert.isTrue(event.from.equals(program.provider.wallet.publicKey));
        assert.strictEqual(event.data.toNumber(), 48);
      } else if (idx === 11111) {
        assert.isTrue(event.from.equals(program.provider.wallet.publicKey));
        assert.strictEqual(event.data.toNumber(), 1234);
      } else if (idx === 24999) {
        assert.isTrue(event.from.equals(program.provider.wallet.publicKey));
        assert.strictEqual(event.data.toNumber(), 99);
      } else {
        assert.isTrue(event.from.equals(PublicKey.default));
        assert.strictEqual(event.data.toNumber(), 0);
      }
    });
  });

  it("Errors when setting an out of bounds index", async () => {
    // Fail to set non existing index.
    await nativeAssert.rejects(
      async () => {
        await program.methods
          .updateLargeAccount(25000, new BN(1))
          .accounts({
            eventQ: eventQ.publicKey,
            from: program.provider.wallet.publicKey,
          })
          .rpc();
      },
      (err) => {
        console.log("err", err);
        return true;
      }
    );
  });
});
