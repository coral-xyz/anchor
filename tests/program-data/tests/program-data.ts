import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { sleep } from '@project-serum/common';
import assert from 'assert';
import { ProgramData } from '../target/types/program_data';

describe('program-data', () => {
  const provider = anchor.Provider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.ProgramData as Program<ProgramData>;
  const programDataAddress = findProgramAddressSync(
    [program.programId.toBytes()],
    new anchor.web3.PublicKey("BPFLoaderUpgradeab1e11111111111111111111111")
  )[0];

  it('Reads ProgramData and sets field', async () => {
    const settings = anchor.web3.Keypair.generate();
    const tx = await program.rpc.setAdminSettings(new anchor.BN(500), {
      accounts: {
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        programData: programDataAddress,
        settings: settings.publicKey
      },
      signers: [settings]
    });
    assert.equal((await program.account.settings.fetch(settings.publicKey)).adminData, 500);

    console.log("Your transaction signature", tx);
  });

  it('Validates constraint on ProgramData', async () => {
    const settings = anchor.web3.Keypair.generate();
    try {
      const authority = anchor.web3.Keypair.generate();
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(authority.publicKey, 10000000000),
        "confirmed"
      );
      await program.rpc.setAdminSettings(new anchor.BN(500), {
        accounts: {
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          programData: programDataAddress,
          settings: settings.publicKey
        },
        signers: [settings, authority]
      });
      assert.ok(false);
    } catch (err) {
      assert.equal(err.code, 143);
      assert.equal(err.msg, "A raw constraint was violated");
    }
  });

  it('Validates that account is ProgramData', async () => {
    const settings = anchor.web3.Keypair.generate();
    try {
      const authority = anchor.web3.Keypair.generate();
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(authority.publicKey, 10000000000),
        "confirmed"
      );
      await program.rpc.setAdminSettings(new anchor.BN(500), {
        accounts: {
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          programData: program.programId,
          settings: settings.publicKey
        },
        signers: [settings, authority]
      });
      assert.ok(false);
    } catch (err) {
      assert.equal(err.code, 173);
      assert.equal(err.msg, "The given account is not a program data account");
    }
  });
});
