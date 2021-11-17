const anchor = require('@project-serum/anchor');
const assert = require('assert');

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
          account1: authority.publicKey,
          account2: anchor.web3.Keypair.generate().publicKey
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
        account1: authority.publicKey,
        account2: authority.publicKey
      },
      signers: [authority]
    });

  });

  it('Emits a ConstraintDup error because account is not a duplicate!', async () => {
    try {
      await program.rpc.withDupConstraintComposite({
        accounts: {
          account1: authority.publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey
          }
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkDupError(err);
    }
  });

  it('Passes the ConstraintDup check because account is a declared duplicate!', async () => {
      await program.rpc.withDupConstraintComposite({
        accounts: {
          account1: authority.publicKey,
          child: {
            childAccount: authority.publicKey
          }
        },
        signers: [authority]
      });
  });

  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    try {
      await program.rpc.withoutDupConstraint({
        accounts: {
          account1: authority.publicKey,
          account2: authority.publicKey
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Passes the ConstraintNoDup check because accounts are not the same!', async () => {
    await program.rpc.withoutDupConstraint({
      accounts: {
        account1: authority.publicKey,
        account2: anchor.web3.Keypair.generate().publicKey
      },
      signers: [authority]
    });
  });
  
  it('Passed a ConstraintNoDup check because duplicate account is inside a CompositeField!', async () => {
    await program.rpc.withoutDupConstraintComposite({
      accounts: {
        account1: authority.publicKey,
        child: {
          childAccount: authority.publicKey
        }
      },
      signers: [authority]
    });
  });

    it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    try {
      await program.rpc.withMissingDupConstraintsThreeAccounts({
        accounts: {
          account1: authority.publicKey,
          account2: authority.publicKey,
          account3: authority.publicKey
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Passes the ConstraintNoDup check because accounts are not the same!', async () => {
    await program.rpc.withMissingDupConstraintsThreeAccounts({
      accounts: {
        account1: authority.publicKey,
        account2: authority.publicKey,
        account3: anchor.web3.Keypair.generate().publicKey
      },
      signers: [authority]
    });
  });

  it('Passes the ConstraintDup check because accounts are declared duplicate!', async () => {
    await program.rpc.withDupConstraintsThreeAccounts({
      accounts: {
        account1: authority.publicKey,
        account2: authority.publicKey,
        account3: authority.publicKey
      },
      signers: [authority]
    });
  });

  it('Emits a ConstraintNoDup error because account is a duplicate!', async () => {
    let otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
    try {
      await program.rpc.withMissingDupConstraintDoubleThreeAccounts({
        accounts: {
          account1: authority.publicKey,
          account2: authority.publicKey,
          account3: authority.publicKey,
          account4: otherDuplicateKey,
          account5: otherDuplicateKey,
          account6: otherDuplicateKey
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Passes the ConstraintNoDup check because account is not a duplicate!', async () => {
    let otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
    await program.rpc.withMissingDupConstraintDoubleThreeAccounts({
      accounts: {
        account1: authority.publicKey,
        account2: authority.publicKey,
        account3: authority.publicKey,
        account4: otherDuplicateKey,
        account5: otherDuplicateKey,
        account6: anchor.web3.Keypair.generate().publicKey
      },
      signers: [authority]
    });
  });

  it('Passes the ConstraintNoDup check because accounts are all immutable!', async () => {
    let otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
    await program.rpc.withoutDupConstraintDoubleThreeAccountsAllImmutable({
      accounts: {
        account1: authority.publicKey,
        account2: authority.publicKey,
        account3: otherDuplicateKey,
        account4: otherDuplicateKey,
        account5: otherDuplicateKey,
        account6: otherDuplicateKey
      },
      signers: [authority]
    });
  });
});
