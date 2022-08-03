const { assert } = require("chai");
const anchor = require("@project-serum/anchor");

describe("optional", () => {
  const provider = anchor.AnchorProvider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  it("Is initialized!", async () => {
    const program = anchor.workspace.Optional;

    const dummy = anchor.web3.Keypair.generate();
  });
});
