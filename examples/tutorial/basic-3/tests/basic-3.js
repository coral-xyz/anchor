const assert = require("assert");
//const anchor = require("@project-serum/anchor");
const anchor = require('/home/armaniferrante/Documents/code/src/github.com/project-serum/anchor/ts');

describe("basic-3", () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  it("Performs CPI from puppet master to puppet", async () => {
    const puppetMaster = anchor.workspace.PuppetMaster;
    const puppet = anchor.workspace.Puppet;

    const newPuppetAccount = new anchor.web3.Account();
    const tx = await puppet.rpc.initialize({
      accounts: {
        puppet: newPuppetAccount.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [newPuppetAccount],
      instructions: [
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: newPuppetAccount.publicKey,
          space: 8 + 8, // Add 8 for the account discriminator.
          lamports: await provider.connection.getMinimumBalanceForRentExemption(
            8 + 8
          ),
          programId: puppet.programId,
        }),
      ],
    });

    let puppetAccount = await puppet.account.puppet(newPuppetAccount.publicKey);
    assert.ok(puppetAccount.data.eq(new anchor.BN(0)));

		await puppetMaster.rpc.pullStrings(new anchor.BN(111), {
				accounts: {
						puppet: newPuppetAccount.publicKey,
						puppetProgram: puppet.programId,
				},
		});
    puppetAccount = await puppet.account.puppet(newPuppetAccount.publicKey);

		assert.ok(puppetAccount.data.eq(new anchor.BN(111)));

		/*
    await puppet.rpc.setData(new anchor.BN(444), {
      accounts: { puppet: newPuppetAccount.publicKey },
    });
    puppetAccount = await puppet.account.puppet(newPuppetAccount.publicKey);

    assert.ok(puppetAccount.data.eq(new anchor.BN(444)));
		*/
  });
});
