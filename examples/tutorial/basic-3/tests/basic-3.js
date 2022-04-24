const assert = require("assert");
const anchor = require("@project-serum/anchor");
const { SystemProgram } = anchor.web3;

describe("basic-3", () => {
  const provider = anchor.AnchorProvider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  it("Performs CPI from puppet master to puppet", async () => {
    const puppetMaster = anchor.workspace.PuppetMaster;
    const puppet = anchor.workspace.Puppet;

    // Initialize a new puppet account.
    const newPuppetAccount = anchor.web3.Keypair.generate();
    
    // The accounts to pass in the initialize rpc
    const accounts = {
      puppet: newPuppetAccount.publicKey,
      user: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    };

    // The signers to pass in the initialize rpc
    const signers = [newPuppetAccount];

    const tx = await puppet.methods.initialize().accounts(accounts).signers(signers).rpc();

    // Invoke the puppet master to perform a CPI to the puppet.
    await puppetMaster.methods.pullStrings(new anchor.BN(111)).account({
      puppet: newPuppetAccount.publicKey,
      puppetProgram: puppet.programId,
    }).rpc();

    // Check the state updated.
    puppetAccount = await puppet.account.data.fetch(newPuppetAccount.publicKey);
    assert.ok(puppetAccount.data.eq(new anchor.BN(111)));
  });
});
