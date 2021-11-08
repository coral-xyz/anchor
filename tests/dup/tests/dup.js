const anchor = require('@project-serum/anchor');
const assert = require('assert');

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
  const program = anchor.workspace.SystemAccounts;
  const authority = program.provider.wallet.payer;

  it('Emits an ConstraintDup error because account is not a duplicate!', async () => {
    try {
      await program.rpc.withDupConstraint({
        accounts: {
          authority: authority.publicKey,
          wallet: anchor.web3.Keypair.generate()
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

  it('Emits an ConstraintNoDup error because account is a duplicate!', async () => {
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

  it('Emits an ConstraintNoDup error because account is a duplicate!', async () => {
    try {
      await program.rpc.withMissingDupConstraint3Accounts({
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

    it('Emits an ConstraintNoDup error because account is a duplicate!', async () => {
      let otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
      try {
        await program.rpc.withMissingDupConstraintDouble3Accounts({
          accounts: {
            myAccount: authority.publicKey,
            rent: authority.publicKey,
            authority: authority.publicKey,
            myAccount_1: otherDuplicateKey,
            rent_1: otherDuplicateKey,
            authority_1: otherDuplicateKey
          },
          signers: [authority]
        });
      } catch (err) {
        checkNoDupError(err);
      }
    });

  });
});
