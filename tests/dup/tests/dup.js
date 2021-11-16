const anchor = require('@project-serum/anchor');
const assert = require('assert');

//TODO[paulx]: add tests for composite fields and adjust tests to test only nodup checks for accounts where at least 1 acc in the pair is mutable

const checkDupError = (err) => {
  const errMsg = "A dup constraint was violated";
  assert.equal(err.toString(), errMsg);
  assert.equal(err.msg, errMsg);
  assert.equal(err.code, 153);
}

const checkNoDupError = (err) => {
  const errMsg = "a nodup constraint was violated";
  assert.equal(err.toString(), errMsg);
  assert.equal(err.msg, errMsg);
  assert.equal(err.code, 154);
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

  it('Passes the ConstraintDup check because account is a duplicate!', async () => {
    await program.rpc.withDupConstraint({
      accounts: {
        authority: authority.publicKey,
        wallet: authority.publicKey
      },
      signers: [authority]
    });

  });

  it('Emits a ConstraintNoDup error because account is a duplicate!', async () => {
    try {
      await program.rpc.withoutDupConstraint({
        accounts: {
          myAccount: authority.publicKey,
          rent: authority.publicKey
        },
        signers: [authority]
      });
    } catch (err) {
      checkNoDupError(err);
    }

  });

  it('Emits a ConstraintNoDup error because account is a duplicate!', async () => {
    try {
      await program.rpc.withMissingDupConstraints3Accounts({
        accounts: {
          myAccount: authority.publicKey,
          rent: authority.publicKey,
          authority: authority.publicKey
        },
        signers: [authority]
      });
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Passes the ConstraintDup check because accounts are duplicate!', async () => {
    await program.rpc.withDupConstraint({
      accounts: {
        myAccount: authority.publicKey,
        rent: authority.publicKey,
        authority: authority.publicKey
      },
      signers: [authority]
    });

    it('Emits a ConstraintNoDup error because account is a duplicate!', async () => {
      let otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
      try {
        await program.rpc.withMissingDupConstraintDouble3Accounts({
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
      } catch (err) {
        checkNoDupError(err);
      }
    });

  });
});
