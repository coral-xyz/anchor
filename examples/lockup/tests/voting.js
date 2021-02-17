const assert = require("assert");
//const anchor = require("@project-serum/anchor");
const anchor = require("/home/armaniferrante/Documents/code/src/github.com/project-serum/anchor/ts");
const serumCmn = require("@project-serum/common");
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const utils = require("./utils");

describe("Governance", () => {
  // Read the provider from the configured environmnet.
  const provider = anchor.Provider.env();

  // Configure the client to use the provider.
  anchor.setProvider(provider);

  const registry = anchor.workspace.Registry;
  const lockup = anchor.workspace.Lockup;
  const voting = anchor.workspace.Voting;
  const majority = anchor.workspace.ThresholdMajority;
  const superMajority = anchor.workspace.ThresholdSuperMajority;

  let mint = null;
  let god = null;

  it("Sets up initial test state", async () => {
    const [_mint, _god] = await serumCmn.createMintAndVault(
      provider,
      new anchor.BN(1000000000000)
    );
    mint = _mint;
    god = _god;
  });

  let registrar = null;
  const member = new anchor.web3.Account();
  let memberAccount = null;
  let poolMint = null;

  it("Setups up stake state", async () => {
    registrar = new anchor.web3.Account();
    const rewardQ = new anchor.web3.Account();
    const withdrawalTimelock = new anchor.BN(4);
    const stakeRate = new anchor.BN(2);
    const rewardQLen = 170;

    // Setup registry program and global state.
    const [
      registrarSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer()],
      registry.programId
    );
    poolMint = await serumCmn.createMint(provider, registrarSigner);
    await registry.state.rpc.new({
      accounts: { lockupProgram: lockup.programId },
    });

    // Create registrar.
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

    const registrarAccount = await registry.account.registrar(
      registrar.publicKey
    );

    // Create member account.
    const [
      memberSigner,
      _nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer(), member.publicKey.toBuffer()],
      registry.programId
    );
    const [mainTx, _balances] = await utils.createBalanceSandbox(
      provider,
      registrarAccount,
      memberSigner
    );
    const [lockedTx, _balancesLocked] = await utils.createBalanceSandbox(
      provider,
      registrarAccount,
      memberSigner
    );

    balances = _balances;
    balancesLocked = _balancesLocked;

    const tx = registry.transaction.createMember(_nonce, {
      accounts: {
        registrar: registrar.publicKey,
        member: member.publicKey,
        beneficiary: provider.wallet.publicKey,
        memberSigner,
        balances,
        balancesLocked,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      instructions: [await registry.account.member.createInstruction(member)],
    });
    const signers = [member, provider.wallet.payer];
    const allTxs = [mainTx, lockedTx, { tx, signers }];
    await provider.sendAll(allTxs);

    memberAccount = await registry.account.member(member.publicKey);

    const depositAmount = new anchor.BN(120);
    await registry.rpc.deposit(depositAmount, {
      accounts: {
        depositor: god,
        depositorAuthority: provider.wallet.publicKey,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        vault: memberAccount.balances.vault,
        beneficiary: provider.wallet.publicKey,
        member: member.publicKey,
      },
    });

    const stakeAmount = new anchor.BN(10);
    await registry.rpc.stake(stakeAmount, false, {
      accounts: {
        // Stake instance.
        registrar: registrar.publicKey,
        rewardEventQ: rewardQ.publicKey,
        poolMint,
        // Member.
        member: member.publicKey,
        beneficiary: provider.wallet.publicKey,
        balances,
        balancesLocked,
        // Program signers.
        memberSigner,
        registrarSigner,
        // Misc.
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });

    const vault = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.vault
    );
    const vaultStake = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.vaultStake
    );
    const spt = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.spt
    );

    assert.ok(vault.amount.eq(new anchor.BN(100)));
    assert.ok(vaultStake.amount.eq(new anchor.BN(20)));
    assert.ok(spt.amount.eq(new anchor.BN(10)));
  });

  // Voting tests start here.

  let governor = new anchor.web3.Account();
  let governorSigner = null;
  let pollQ = new anchor.web3.Account();
  let proposalQ = new anchor.web3.Account();
  let time = new anchor.BN(60);
  const pollPrice = new anchor.BN(10 * 10 ** 6);
  const proposalPrice = new anchor.BN(10000 * 10 ** 6);
  let governorAccount = null;
  const adjudicator = new anchor.web3.Account();
  const recursiveAdjudicator = new anchor.web3.Account();

  it("Creates a governor", async () => {
    const [
      _governorSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [governor.publicKey.toBuffer()],
      voting.programId
    );
    governorSigner = _governorSigner;
    await voting.rpc.createGovernor(
      adjudicator.publicKey,
      recursiveAdjudicator.publicKey,
      mint,
      time,
      nonce,
      pollPrice,
      proposalPrice,
      150,
      {
        accounts: {
          governor: governor.publicKey,
          pollQ: pollQ.publicKey,
          proposalQ: proposalQ.publicKey,
          registrar: registrar.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        instructions: [
          await voting.account.govQueue.createInstruction(pollQ, 8250),
          await voting.account.govQueue.createInstruction(proposalQ, 8250),
          await voting.account.governor.createInstruction(governor),
        ],
        signers: [pollQ, proposalQ, governor],
      }
    );

    governorAccount = await voting.account.governor(governor.publicKey);

    assert.ok(governorAccount.adjudicator.equals(adjudicator.publicKey));
    assert.ok(governorAccount.registrar.equals(registrar.publicKey));
    assert.ok(governorAccount.nonce == nonce);
    assert.ok(governorAccount.time.eq(time));
    assert.ok(governorAccount.pollQ.equals(pollQ.publicKey));
    assert.ok(governorAccount.proposalQ.equals(proposalQ.publicKey));
    assert.ok(governorAccount.pollPrice.eq(pollPrice));
    assert.ok(governorAccount.proposalPrice.eq(proposalPrice));
  });

  const poll = new anchor.web3.Account();
  const pollVault = new anchor.web3.Account();
  let pollSigner = null;

  it("Creates a poll", async () => {
    const msg = "This is a test";
    const options = ["asdf", "qwer", "zcxv"];
    const endTs = new anchor.BN(Date.now() / 1000 + 30);
    const [_pollSigner, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [poll.publicKey.toBuffer()],
      voting.programId
    );
    pollSigner = _pollSigner;

    await voting.rpc.createPoll(msg, options, endTs, nonce, {
      accounts: {
        poll: poll.publicKey,
        governor: governor.publicKey,
        pollQ: pollQ.publicKey,
        depositor: god,
        depositorAuthority: voting.provider.wallet.publicKey,
        vault: pollVault.publicKey,
        pollSigner,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
      instructions: [
        await voting.account.poll.createInstruction(poll, 2000),
        ...(await serumCmn.createTokenAccountInstrs(
          provider,
          pollVault.publicKey,
          mint,
          pollSigner
        )),
      ],
      signers: [poll, pollVault],
    });

    const pollAccount = await voting.account.poll(poll.publicKey);

    assert.ok(pollAccount.governor.equals(governor.publicKey));
    assert.ok(pollAccount.msg === "This is a test");
    assert.ok(pollAccount.startTs.gt(new anchor.BN(0)));
    assert.ok(pollAccount.endTs.eq(endTs));
    assert.deepEqual(pollAccount.options, options);
    assert.ok(pollAccount.nonce === nonce);
    assert.ok(pollAccount.vault.equals(pollVault.publicKey));

    const pollQueue = await voting.account.govQueue(governorAccount.pollQ);
    assert.ok(pollQueue.proposals[0].equals(poll.publicKey));
  });

  it("Votes for a poll", async () => {
    const [vote, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [poll.publicKey.toBuffer(), member.publicKey.toBuffer()],
      voting.programId
    );
    await voting.rpc.votePoll(1, nonce, {
      accounts: {
        vote,
        governor: governor.publicKey,
        poll: poll.publicKey,
        stake: {
          member: member.publicKey,
          beneficiary: voting.provider.wallet.publicKey,
          memberSpt: memberAccount.balances.spt,
          memberSptLocked: memberAccount.balancesLocked.spt,
        },
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: new anchor.web3.PublicKey(
          "11111111111111111111111111111111"
        ),
      },
    });

    const pollAccount = await voting.account.poll(poll.publicKey);
    assert.deepEqual(
      pollAccount.voteWeights.map((v) => v.toNumber()),
      [0, 10, 0]
    );

    const voteAccount = await voting.account.vote(vote);
    assert.ok(voteAccount.burned);
    assert.ok(voteAccount.selector, 1);
    assert.ok(voteAccount.account.equals(poll.publicKey));
    assert.ok(voteAccount.member.equals(member.publicKey));
  });

  const proposal = new anchor.web3.Account();
  const proposalVault = new anchor.web3.Account();

  it("Creates a proposal", async () => {
    const [
      proposalSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [proposal.publicKey.toBuffer()],
      voting.programId
    );
    const tx = {
      programId: anchor.web3.SYSVAR_RENT_PUBKEY,
      didExecute: false,
      accounts: [
        {
          pubkey: anchor.web3.SYSVAR_RENT_PUBKEY,
          isSigner: true,
          isWritable: true,
        },
      ],
      data: Buffer.from([1, 2, 3, 4]),
    };
    await voting.rpc.createProposal("Testing proposal", tx, nonce, {
      accounts: {
        proposal: proposal.publicKey,
        governor: governor.publicKey,
        proposalQ: proposalQ.publicKey,
        vault: proposalVault.publicKey,
        proposalSigner,
        poolMint,
        registrar: registrar.publicKey,
        depositor: god,
        depositorAuthority: voting.provider.wallet.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
      signers: [proposal, proposalVault],
      instructions: [
        await voting.account.proposal.createInstruction(proposal, 2000),
        ...(await serumCmn.createTokenAccountInstrs(
          provider,
          proposalVault.publicKey,
          mint,
          proposalSigner
        )),
      ],
    });

    const proposalAccount = await voting.account.proposal(proposal.publicKey);

    assert.ok(proposalAccount.msg === "Testing proposal");
    assert.ok(proposalAccount.nonce === nonce);
    assert.ok(!proposalAccount.burned);
    assert.ok(proposalAccount.endTs.sub(proposalAccount.startTs));
    assert.ok(proposalAccount.voteYes.toNumber() === 0);
    assert.ok(proposalAccount.voteNo.toNumber() === 0);
    assert.ok(proposalAccount.proposer.equals(provider.wallet.publicKey));
    assert.ok(proposalAccount.tx.programId.equals(tx.programId));
    assert.ok(proposalAccount.tx.didExecute === false);
    assert.ok(proposalAccount.stakeTokenSupply.toNumber(), 10);
    assert.ok(
      proposalAccount.tx.accounts[0].pubkey.equals(tx.accounts[0].pubkey)
    );
    assert.ok(
      proposalAccount.tx.accounts[0].isSigner === tx.accounts[0].isSigner
    );
    assert.ok(
      proposalAccount.tx.accounts[0].isWritable === tx.accounts[0].isWritable
    );
    assert.ok(proposalAccount.tx.data.equals(tx.data));
  });

  it("Votes for a proposal", async () => {
    const [vote, nonce] = await anchor.web3.PublicKey.findProgramAddress(
      [proposal.publicKey.toBuffer(), member.publicKey.toBuffer()],
      voting.programId
    );
    await voting.rpc.voteProposal(true, nonce, {
      accounts: {
        governor: governor.publicKey,
        proposal: proposal.publicKey,
        vote,
        stake: {
          member: member.publicKey,
          beneficiary: voting.provider.wallet.publicKey,
          memberSpt: memberAccount.balances.spt,
          memberSptLocked: memberAccount.balancesLocked.spt,
        },
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: new anchor.web3.PublicKey(
          "11111111111111111111111111111111"
        ),
      },
    });
    const voteAccount = await voting.account.vote(vote);
    assert.ok(voteAccount.selector === 1);
    assert.ok((voteAccount.burned = true));
    assert.ok(voteAccount.member.equals(member.publicKey));
    assert.ok(voteAccount.account.equals(proposal.publicKey));

    proposalAccount = await voting.account.proposal(proposal.publicKey);
    assert.ok(proposalAccount.voteYes.toNumber() === 10);
    assert.ok(proposalAccount.voteNo.toNumber() === 0);
  });
});
