const anchor = require('@project-serum/anchor')
const PublicKey = anchor.web3.PublicKey
const BN = anchor.BN
const assert = require('assert')

describe('zero-copy', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env())

  const program = anchor.workspace.ZeroCopy
  const programCpi = anchor.workspace.ZeroCpi

  const foo = anchor.web3.Keypair.generate()
  describe('Creates and updates a zero copy account', () => {
    // The update tests need to run after the create test

    it('Is creates a zero copy account', async () => {
      await program.rpc.createFoo({
        accounts: {
          foo: foo.publicKey,
          authority: program.provider.wallet.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        instructions: [await program.account.foo.createInstruction(foo)],
        signers: [foo],
      })
      const account = await program.account.foo.fetch(foo.publicKey)
      assert.ok(
        JSON.stringify(account.authority.toBuffer()) ===
          JSON.stringify(program.provider.wallet.publicKey.toBuffer())
      )
      assert.ok(account.data.toNumber() === 0)
      assert.ok(account.secondData.toNumber() === 0)
      assert.ok(
        JSON.stringify(account.secondAuthority) ===
          JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
      )
    })

    it('Updates a zero copy account field', async () => {
      await program.rpc.updateFoo(new BN(1234), {
        accounts: {
          foo: foo.publicKey,
          authority: program.provider.wallet.publicKey,
        },
      })
  
      const account = await program.account.foo.fetch(foo.publicKey)
  
      assert.ok(
        JSON.stringify(account.authority.toBuffer()) ===
          JSON.stringify(program.provider.wallet.publicKey.toBuffer())
      )
      assert.ok(account.data.toNumber() === 1234)
      assert.ok(account.secondData.toNumber() === 0)
      assert.ok(
        JSON.stringify(account.secondAuthority) ===
          JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
      )
    })

    it('Updates a second zero copy account field', async () => {
      await program.rpc.updateFooSecond(new BN(55), {
        accounts: {
          foo: foo.publicKey,
          secondAuthority: program.provider.wallet.publicKey,
        },
      })
  
      const account = await program.account.foo.fetch(foo.publicKey)
  
      assert.ok(
        JSON.stringify(account.authority.toBuffer()) ===
          JSON.stringify(program.provider.wallet.publicKey.toBuffer())
      )
      assert.ok(account.data.toNumber() === 1234)
      assert.ok(account.secondData.toNumber() === 55)
      assert.ok(
        JSON.stringify(account.secondAuthority) ===
          JSON.stringify([...program.provider.wallet.publicKey.toBuffer()])
      )
    })

    it('Updates a zero copy account field via cpi', async () => {
      /**
       * assert that account.data does not equal to 420 before update
       * so tests don't accidentally pass
       */
      //
      {
        const fooAccountAfterCpi = await program.account.foo.fetch(foo.publicKey)
        assert.notEqual(fooAccountAfterCpi.data.toNumber(), 420)
      }

      let listener = null;
      let [event, slot] = await new Promise((resolve, _reject) => {
        listener = programCpi.addEventListener("MyEvent", (event, slot) => {
          resolve([event, slot]);
        });
        // Check zero_copy CPI
        programCpi.rpc.updateFooViaCpi(new BN(420), {
          accounts: {
            authority: program.provider.wallet.publicKey,
            foo: foo.publicKey,
            zeroCopyProgram: program.programId,
          },
        })
      });
      await programCpi.removeEventListener(listener);
      const fooAccountAfterCpi = await program.account.foo.fetch(foo.publicKey)
      assert.equal(fooAccountAfterCpi.data.toNumber(), 420)
      assert.ok(slot > 0);
      /**
       * we use an event to assert that if we take a reference
       * to foo before we update foo via the cpi,
       * the reference will automatically point
       * to the updated data after the cpi returned
       */ 
      assert.equal(event.data.toNumber(), 420);
    })
  })

  describe('Creates and updates an associated zero copy account', () => {
    // The update tests need to run after the create test

    it('Creates an associated zero copy account', async () => {
      await program.rpc.createBar({
        accounts: {
          bar: (
            await PublicKey.findProgramAddress(
              [program.provider.wallet.publicKey.toBuffer(), foo.publicKey.toBuffer()],
              program.programId
            )
          )[0],
          authority: program.provider.wallet.publicKey,
          foo: foo.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      })
  
      const bar = (
        await PublicKey.findProgramAddress(
          [program.provider.wallet.publicKey.toBuffer(), foo.publicKey.toBuffer()],
          program.programId
        )
      )[0]
      const barAccount = await program.account.bar.fetch(bar)
      assert.ok(barAccount.authority.equals(program.provider.wallet.publicKey))
      assert.ok(barAccount.data.toNumber() === 0)
    })

    it('Updates an associated zero copy account', async () => {
      const bar = (
        await PublicKey.findProgramAddress(
          [program.provider.wallet.publicKey.toBuffer(), foo.publicKey.toBuffer()],
          program.programId
        )
      )[0]
      await program.rpc.updateBar(new BN(99), {
        accounts: {
          bar,
          authority: program.provider.wallet.publicKey,
          foo: foo.publicKey,
        },
      })
      const barAccount = await program.account.bar.fetch(bar)
      assert.ok(barAccount.authority.equals(program.provider.wallet.publicKey))
      assert.ok(barAccount.data.toNumber() === 99)
      // Check zero_copy CPI
      await programCpi.rpc.checkCpi(new BN(1337), {
        accounts: {
          bar,
          authority: program.provider.wallet.publicKey,
          foo: foo.publicKey,
          zeroCopyProgram: program.programId,
        },
      })
      const barAccountAfterCpi = await program.account.bar.fetch(bar)
      assert.ok(barAccountAfterCpi.authority.equals(program.provider.wallet.publicKey))
      assert.ok(barAccountAfterCpi.data.toNumber() === 1337)
    })
  }) 

  describe('Creates and updates a large event queue', () => {
    // The update tests need to run after the create test

    const eventQ = anchor.web3.Keypair.generate()
    const size = 1000000 + 8 // Account size in bytes.
  
    it('Creates a large event queue', async () => {
      await program.rpc.createLargeAccount({
        accounts: {
          eventQ: eventQ.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        instructions: [await program.account.eventQ.createInstruction(eventQ, size)],
        signers: [eventQ],
      })
      const account = await program.account.eventQ.fetch(eventQ.publicKey)
      assert.ok(account.events.length === 25000)
      account.events.forEach((event) => {
        assert.ok(event.from.equals(PublicKey.default))
        assert.ok(event.data.toNumber() === 0)
      })
    })


    it('Updates a large event queue', async () => {
      // Set index 0.
      await program.rpc.updateLargeAccount(0, new BN(48), {
        accounts: {
          eventQ: eventQ.publicKey,
          from: program.provider.wallet.publicKey,
        },
      })
      // Verify update.
      let account = await program.account.eventQ.fetch(eventQ.publicKey)
      assert.ok(account.events.length === 25000)
      account.events.forEach((event, idx) => {
        if (idx === 0) {
          assert.ok(event.from.equals(program.provider.wallet.publicKey))
          assert.ok(event.data.toNumber() === 48)
        } else {
          assert.ok(event.from.equals(PublicKey.default))
          assert.ok(event.data.toNumber() === 0)
        }
      })

      // Set index 11111.
      await program.rpc.updateLargeAccount(11111, new BN(1234), {
        accounts: {
          eventQ: eventQ.publicKey,
          from: program.provider.wallet.publicKey,
        },
      })
      // Verify update.
      account = await program.account.eventQ.fetch(eventQ.publicKey)
      assert.ok(account.events.length === 25000)
      account.events.forEach((event, idx) => {
        if (idx === 0) {
          assert.ok(event.from.equals(program.provider.wallet.publicKey))
          assert.ok(event.data.toNumber() === 48)
        } else if (idx === 11111) {
          assert.ok(event.from.equals(program.provider.wallet.publicKey))
          assert.ok(event.data.toNumber() === 1234)
        } else {
          assert.ok(event.from.equals(PublicKey.default))
          assert.ok(event.data.toNumber() === 0)
        }
      })

      // Set last index.
      await program.rpc.updateLargeAccount(24999, new BN(99), {
        accounts: {
          eventQ: eventQ.publicKey,
          from: program.provider.wallet.publicKey,
        },
      })
      // Verify update.
      account = await program.account.eventQ.fetch(eventQ.publicKey)
      assert.ok(account.events.length === 25000)
      account.events.forEach((event, idx) => {
        if (idx === 0) {
          assert.ok(event.from.equals(program.provider.wallet.publicKey))
          assert.ok(event.data.toNumber() === 48)
        } else if (idx === 11111) {
          assert.ok(event.from.equals(program.provider.wallet.publicKey))
          assert.ok(event.data.toNumber() === 1234)
        } else if (idx === 24999) {
          assert.ok(event.from.equals(program.provider.wallet.publicKey))
          assert.ok(event.data.toNumber() === 99)
        } else {
          assert.ok(event.from.equals(PublicKey.default))
          assert.ok(event.data.toNumber() === 0)
        }
      })
    })

    it('Errors when setting an out of bounds index', async () => {
      // Fail to set non existing index.
      await assert.rejects(
        async () => {
          await program.rpc.updateLargeAccount(25000, new BN(1), {
            accounts: {
              eventQ: eventQ.publicKey,
              from: program.provider.wallet.publicKey,
            },
          })
        },
        (err) => {
          console.log('err', err)
          return true
        }
      )
    })
  })
})
