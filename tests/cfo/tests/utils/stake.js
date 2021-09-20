const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const utils = require("../../deps/stake/tests/utils");

const lockup = anchor.workspace.Lockup;
const registry = anchor.workspace.Registry;
const provider = anchor.Provider.env();

const withdrawalTimelock = new anchor.BN(4);
const stakeRate = new anchor.BN(2);
const rewardQLen = 170;

const WHITELIST_SIZE = 10;

async function setupStakePool(mint, god) {
	const registrar = new anchor.web3.Account();
	const rewardQ = new anchor.web3.Account();

  // Registry genesis.
  const [
    registrarSigner,
    nonce,
  ] = await anchor.web3.PublicKey.findProgramAddress(
    [registrar.publicKey.toBuffer()],
    registry.programId
  );
  const poolMint = await serumCmn.createMint(provider, registrarSigner);

  try {
    // Init registry.
    await registry.state.rpc.new({
      accounts: { lockupProgram: lockup.programId },
    });

    // Init lockup.
    await lockup.state.rpc.new({
      accounts: {
        authority: provider.wallet.publicKey,
      },
    });
  } catch (err) {
    // Skip errors for convenience when developing locally,
    // since the state constructors can only be called once.
  }

  // Initialize stake pool.
  await registry.rpc.initialize(
    mint,
    provider.wallet.publicKey,
    nonce,
    withdrawalTimelock,
    stakeRate,
    rewardQLen,
    {
      accounts: {
        registrar: registrar.publicKey,
        poolMint,
        rewardEventQ: rewardQ.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [registrar, rewardQ],
      instructions: [
        await registry.account.registrar.createInstruction(registrar),
        await registry.account.rewardQueue.createInstruction(rewardQ, 8250),
      ],
    }
  );
		return {
				registrar: registrar.publicKey,
				poolMint,
				rewardEventQ: rewardQ.publicKey,
		};
}

module.exports = {
  setupStakePool,
};
