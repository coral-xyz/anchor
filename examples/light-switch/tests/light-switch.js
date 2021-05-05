const assert = require("assert");
const anchor = require('@project-serum/anchor');

describe('initialize light switch', () => {

  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  // initialize programs
  const light = anchor.workspace.Light;
  const lightSwitch = anchor.workspace.LightSwitch;
  let switchSigner = null;
  let nonce = null
  const switchAccount = new anchor.web3.Account();

  it('Is initialized!', async () => {


    // obtain PDA for switch
    let [
      _switchSigner,
      _nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [switchAccount.publicKey.toBuffer()],
      lightSwitch.programId
    );
    switchSigner = _switchSigner;
    nonce = _nonce

    // initialize light with switch account
    await light.state.rpc.new({
      accounts: {
        switch: switchAccount.publicKey,
      }
    });

    // initialize light with authority and switch account
    // both light and light switch share the same switch account key.
    await lightSwitch.state.rpc.new({
          accounts: {
            authority: provider.wallet.publicKey,
            switch: switchAccount.publicKey,
          },
    });

    // Fetch the state struct from the network.
    const state = await light.state();
    // light is initialized, and not on
    assert.ok(!state.isLightOn);
  });

    it("Light switch is able to turn on the light", async () => {

      await lightSwitch.state.rpc.flip(nonce, {
        accounts: {
          authority: provider.wallet.publicKey, // only authority can perform the instruction
          switch: switchAccount.publicKey, // pass in switch account
          cpiState: await light.state.address(), // current state of light
          lightProgram: light.programId, // light program ID
        },
        signers:[switchAccount] // signed by switch account

      });
      const state = await light.state();
      assert.ok(state.isLightOn); // light is on
    });

    it("Random switch is not able to turn off the light", async () => {
      const fakeSwitchAccount = new anchor.web3.Account();
      let [
          _fakeSigner,
          _nonce,
        ] = await anchor.web3.PublicKey.findProgramAddress(
          [fakeSwitchAccount.publicKey.toBuffer()],
          lightSwitch.programId
        );

    try{
      await lightSwitch.state.rpc.flip(_nonce, {
        accounts: {
          authority: provider.wallet.publicKey, // only authority can perform the instruction
          switch: switchAccount.publicKey, // pass in switch account
          cpiState: await light.state.address(), // current state of light
          lightProgram: light.programId, // light program ID
        },
        signers:[fakeSwitchAccount] // sign by new fake account
      });
    } catch (err) {
    }
      const state = await light.state();
      assert.ok(state.isLightOn); // light is still on
    });



});
