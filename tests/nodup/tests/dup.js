const anchor = require('@project-serum/anchor');
const assert = require('assert');

const checkNoDupError = (err) => {
  const errMsg = "A nodup constraint was violated";
  assert.equal(err.toString(), errMsg);
  assert.equal(err.msg, errMsg);
  assert.equal(err.code, 154);
}

describe('dup_error', () => {
  anchor.setProvider(anchor.Provider.local());
  const program = anchor.workspace.Dup;
  const authority = program.provider.wallet.payer;

  it('Passes because account is a declared duplicate', async () => {
    await program.rpc.withDupConstraint({
      accounts: {
        account1: authority.publicKey,
        account2: authority.publicKey
      },
      signers: [authority]
    });
  })

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
  
  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    try {
      await program.rpc.withoutDupConstraintComposite({
        accounts: {
          account1: authority.publicKey,
          child: {
            childAccount: authority.publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey
          }
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch(err) {
      checkNoDupError(err);
    }
  });

  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    try {
      await program.rpc.withoutDupConstraintCompositeReverse({
        accounts: {
          account1: authority.publicKey,
          child: {
            childAccount: authority.publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey
          }
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch(err) {
      checkNoDupError(err);
    }
  });

  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    try {
      await program.rpc.withoutDupConstraintTwoComposites({
        accounts: {
          account: anchor.web3.Keypair.generate().publicKey,
          childTwo: {
            childAccount: authority.publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey
          },
          child: {
            childAccount: authority.publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey
          }
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch(err) {
      checkNoDupError(err);
    }
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
    const otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
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
    const otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
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
    const otherDuplicateKey = anchor.web3.Keypair.generate().publicKey;
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

  it('Passes the ConstraintNoDup check because accounts are all unique!', async () => {
    await program.rpc.nestedChildren({
      accounts: {
        accountOne: {
          account1: anchor.web3.Keypair.generate().publicKey,
          account2: anchor.web3.Keypair.generate().publicKey,
          account3: anchor.web3.Keypair.generate().publicKey,
        },
        accountTwo: {
          account1: anchor.web3.Keypair.generate().publicKey,
          account2: anchor.web3.Keypair.generate().publicKey,
        },
        accountThree: {
          account1: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
          }
        },
        accountFour: {
          account: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
          },
          childTwo: {
            childAccount:anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
          }
        },
        accountFive: anchor.web3.Keypair.generate().publicKey,
      },
      signers: [authority]
    });
  });

  it('Passes the ConstraintNoDup check because accounts are all unique or declared duplicate!', async () => {
    const keyOne = anchor.web3.Keypair.generate().publicKey;
    await program.rpc.nestedChildren({
      accounts: {
        accountOne: {
          account1: keyOne,
          account2: keyOne,
          account3: keyOne
        },
        accountTwo: {
          account1: anchor.web3.Keypair.generate().publicKey,
          account2: anchor.web3.Keypair.generate().publicKey,
        },
        accountThree: {
          account1: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
          }
        },
        accountFour: {
          account: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
          },
          childTwo: {
            childAccount:anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
          }
        },
        accountFive: anchor.web3.Keypair.generate().publicKey,
      },
      signers: [authority]
    });
  });

  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    const keyOne = anchor.web3.Keypair.generate().publicKey;
    try {
      await program.rpc.nestedChildren({
        accounts: {
          accountOne: {
            account1: keyOne,
            account2: keyOne,
            account3: keyOne
          },
          accountTwo: {
            account1: anchor.web3.Keypair.generate().publicKey,
            account2: anchor.web3.Keypair.generate().publicKey,
          },
          accountThree: {
            account1: anchor.web3.Keypair.generate().publicKey,
            child: {
              childAccount: anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
            }
          },
          accountFour: {
            account: anchor.web3.Keypair.generate().publicKey,
            child: {
              childAccount: anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
            },
            childTwo: {
              childAccount:anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
            }
          },
          accountFive: keyOne,
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    const keyOne = anchor.web3.Keypair.generate().publicKey;
    try {
      await program.rpc.nestedChildren({
        accounts: {
          accountOne: {
            account1: keyOne,
            account2: keyOne,
            account3: keyOne
          },
          accountTwo: {
            account1: anchor.web3.Keypair.generate().publicKey,
            account2: anchor.web3.Keypair.generate().publicKey,
          },
          accountThree: {
            account1: anchor.web3.Keypair.generate().publicKey,
            child: {
              childAccount: anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
            }
          },
          accountFour: {
            account: anchor.web3.Keypair.generate().publicKey,
            child: {
              childAccount: anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: keyOne,
            },
            childTwo: {
              childAccount:anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
            }
          },
          accountFive: anchor.web3.Keypair.generate().publicKey,
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Emits a ConstraintNoDup error because account is a mutable duplicate!', async () => {
    const keyOne = anchor.web3.Keypair.generate().publicKey;
    try {
      await program.rpc.nestedChildren({
        accounts: {
          accountOne: {
            account1: keyOne,
            account2: keyOne,
            account3: keyOne
          },
          accountTwo: {
            account1: anchor.web3.Keypair.generate().publicKey,
            account2: anchor.web3.Keypair.generate().publicKey,
          },
          accountThree: {
            account1: anchor.web3.Keypair.generate().publicKey,
            child: {
              childAccount: anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: keyOne
            }
          },
          accountFour: {
            account: anchor.web3.Keypair.generate().publicKey,
            child: {
              childAccount: anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
            },
            childTwo: {
              childAccount:anchor.web3.Keypair.generate().publicKey,
              anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
            }
          },
          accountFive: anchor.web3.Keypair.generate().publicKey,
        },
        signers: [authority]
      });
      assert.ok(false);
    } catch (err) {
      checkNoDupError(err);
    }
  });

  it('Passes the ConstraintNoDup check because accounts are all unique, immutable, or declared duplicate!', async () => {
    const keyOne = anchor.web3.Keypair.generate().publicKey;
    await program.rpc.nestedChildren({
      accounts: {
        accountOne: {
          account1: keyOne,
          account2: keyOne,
          account3: anchor.web3.Keypair.generate().publicKey
        },
        accountTwo: {
          account1: anchor.web3.Keypair.generate().publicKey,
          account2: anchor.web3.Keypair.generate().publicKey,
        },
        accountThree: {
          account1: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: anchor.web3.Keypair.generate().publicKey,
          }
        },
        accountFour: {
          account: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: keyOne,
          },
          childTwo: {
            childAccount:anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: keyOne,
          }
        },
        accountFive: anchor.web3.Keypair.generate().publicKey,
      },
      signers: [authority]
    });
  });

  it('Passes the ConstraintNoDup check because accounts are all unique or immutable!', async () => {
    const keyOne = anchor.web3.Keypair.generate().publicKey;
    const keyTwo = anchor.web3.Keypair.generate().publicKey;

    await program.rpc.nestedChildren({
      accounts: {
        accountOne: {
          account1: keyOne,
          account2: keyOne,
          account3: anchor.web3.Keypair.generate().publicKey
        },
        accountTwo: {
          account1: keyOne,
          account2: anchor.web3.Keypair.generate().publicKey,
        },
        accountThree: {
          account1: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: keyTwo,
            anotherChildAccount: keyOne,
          }
        },
        accountFour: {
          account: anchor.web3.Keypair.generate().publicKey,
          child: {
            childAccount: anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: keyTwo,
          },
          childTwo: {
            childAccount:anchor.web3.Keypair.generate().publicKey,
            anotherChildAccount: keyTwo,
          }
        },
        accountFive: keyTwo
      },
      signers: [authority]
    });
  });
});
