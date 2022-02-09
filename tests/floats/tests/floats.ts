import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Floats } from '../target/types/floats';
import assert from 'assert';

describe('floats', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Floats as Program<Floats>;

  it("Creates an account with float data", async () => {
    const accountKeypair = anchor.web3.Keypair.generate();

    await program.rpc.create(1111.1111, 9999.9999, {
      accounts: {
        account: accountKeypair.publicKey,
        authority: anchor.getProvider().wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [accountKeypair],
    });

    const account = await program.account.floatDataAccount.fetch(accountKeypair.publicKey);

    assert.strictEqual(account.dataF32, 1111.1111);
    assert.strictEqual(account.dataF64, 9999.9999);
  });

  it("Updates an account with float data", async () => {
    const accountKeypair = anchor.web3.Keypair.generate();

    await program.rpc.create(1111.1111, 9999.9999, {
      accounts: {
        account: accountKeypair.publicKey,
        authority: anchor.getProvider().wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [accountKeypair],
    });

    let account = await program.account.floatDataAccount.fetch(accountKeypair.publicKey);

    await program.rpc.update(2222.2222, 8888.8888, {
      accounts: {
        account: accountKeypair.publicKey,
      },
    });

    account = await program.account.floatDataAccount.fetch(accountKeypair.publicKey);

    assert.strictEqual(account.dataF32, 2222.2222);
    assert.strictEqual(account.dataF64, 8888.8888);
  });
});
