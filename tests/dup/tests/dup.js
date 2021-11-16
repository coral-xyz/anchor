const anchor = require('@project-serum/anchor');
const assert = require('assert');

//TODO[paulx]: add tests for composite fields and adjust tests to test only nodup checks for accounts where at least 1 acc in the pair is mutable

const checkDupError = (err) => {
  const errMsg = "A dup constraint was violated";
  //assert.equal(err.toString(), errMsg);
  //assert.equal(err.msg, errMsg);
  assert.equal(err.code, 154);
}

const checkNoDupError = (err) => {
  const errMsg = "A nodup constraint was violated";
  assert.equal(err.toString(), errMsg);
  assert.equal(err.msg, errMsg);
  assert.equal(err.code, 155);
}

describe('dup_error', () => {
  anchor.setProvider(anchor.Provider.local());
  const program = anchor.workspace.Dup;
  const authority = program.provider.wallet.payer;

  it('Emits a ConstraintDup error because account is not a duplicate!', async () => {
    try {
      await program.rpc.withDupConstraint({
        accounts: {
          authority: authority.publicKey,
          wallet: anchor.web3.Keypair.generate().publicKey
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkDupError(err);
    }
  });

  it('Passes the ConstraintDup check because account is a declared duplicate!', async () => {
    await program.rpc.withDupConstraint({
      accounts: {
        authority: authority.publicKey,
        wallet: authority.publicKey
      },
      signers: [authority]
    });

  });

  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    try {
      await program.rpc.withoutDupConstraint({
        accounts: {
          myAccount: authority.publicKey,
          rent: authority.publicKey
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

    it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    try {
      await program.rpc.withMissingDupConstraintsThreeAccounts({
        accounts: {
          myAccount: authority.publicKey,
          rent: authority.publicKey,
          authority: authority.publicKey
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Passes the ConstraintDup check because accounts are declared duplicate!', async () => {
    await program.rpc.withDupConstraintsThreeAccounts({
      accounts: {
        myAccount: authority.publicKey,
        rent: authority.publicKey,
        authority: authority.publicKey
      },
      signers: [authority]
    });
  });

    it('Emits a ConstraintNoDup error because account is a duplicate!', async () => {
      let otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
      try {
        await program.rpc.withMissingDupConstraintDoubleThreeAccounts({
          accounts: {
            myAccount: authority.publicKey,
            rent: authority.publicKey,
            authority: authority.publicKey,
            myAccount1: otherDuplicateKey,
            rent1: otherDuplicateKey,
            authority1: otherDuplicateKey
          },
          signers: [authority]
        });
        assert.ok(false);
      } catch (err) {
        checkNoDupError(err);
      }
    });
});
