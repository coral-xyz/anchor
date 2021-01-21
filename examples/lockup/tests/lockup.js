const assert = require("assert");
const anchor = require('@project-serum/anchor');
const serumCmn = require("@project-serum/common");
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const utils = require("./utils");

describe("Lockup and Registry", () => {
  const provider = anchor.Provider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const lockup = anchor.workspace.Lockup;
  const registry = anchor.workspace.Registry;

  const safe = new anchor.web3.Account();
  const whitelist = new anchor.web3.Account();

  let mint = null;
  let god = null;

  it("Sets up initial test state", async () => {
    const [_mint, _god] = await serumCmn.createMintAndVault(
      provider,
      new anchor.BN(1000000)
    );
    mint = _mint;
    god = _god;
  });

  it("Is initialized!", async () => {
    await lockup.rpc.initialize(provider.wallet.publicKey, {
      accounts: {
        safe: safe.publicKey,
        whitelist: whitelist.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [safe, whitelist],
      instructions: [
        await lockup.account.safe.createInstruction(safe),
        await lockup.account.whitelist.createInstruction(whitelist, 1000),
      ],
    });
    const safeAccount = await lockup.account.safe(safe.publicKey);
    const whitelistAccount = await lockup.account.whitelist(
      whitelist.publicKey
    );

    assert.ok(safeAccount.authority.equals(provider.wallet.publicKey));
    assert.ok(safeAccount.whitelist.equals(whitelist.publicKey));

    assert.ok(whitelistAccount.safe.equals(safe.publicKey));
    assert.ok(whitelistAccount.entries.length === 0);
  });

  it("Sets a new authority", async () => {
    const newAuthority = new anchor.web3.Account();
    await lockup.rpc.setAuthority(newAuthority.publicKey, {
      accounts: {
        authority: provider.wallet.publicKey,
        safe: safe.publicKey,
      },
    });

    let safeAccount = await lockup.account.safe(safe.publicKey);
    assert.ok(safeAccount.authority.equals(newAuthority.publicKey));

    await lockup.rpc.setAuthority(provider.wallet.publicKey, {
      accounts: {
        authority: newAuthority.publicKey,
        safe: safe.publicKey,
      },
      signers: [newAuthority],
    });

    safeAccount = await lockup.account.safe(safe.publicKey);
    assert.ok(safeAccount.authority.equals(provider.wallet.publicKey));
  });

  let e0 = null;
  let e1 = null;
  let e2 = null;
  let e3 = null;
  let e4 = null;

  it("Adds to the whitelist", async () => {
    const generateEntry = async () => {
      let programId = new anchor.web3.Account().publicKey;
      let instance = new anchor.web3.Account().publicKey;
      let [_, nonce] = await anchor.web3.PublicKey.findProgramAddress(
        [instance.toBuffer()],
        programId
      );
      return {
        programId,
        instance,
        nonce,
      };
    };
    e0 = await generateEntry();
    e1 = await generateEntry();
    e2 = await generateEntry();
    e3 = await generateEntry();
    e4 = await generateEntry();
    const e5 = await generateEntry();

    const accounts = {
      authority: provider.wallet.publicKey,
      safe: safe.publicKey,
      whitelist: whitelist.publicKey,
    };

    await lockup.rpc.whitelistAdd(e0, { accounts });

    let whitelistAccount = await lockup.account.whitelist(whitelist.publicKey);

    assert.ok(whitelistAccount.entries.length === 1);
    assert.deepEqual(whitelistAccount.entries, [e0]);

    await lockup.rpc.whitelistAdd(e1, { accounts });
    await lockup.rpc.whitelistAdd(e2, { accounts });
    await lockup.rpc.whitelistAdd(e3, { accounts });
    await lockup.rpc.whitelistAdd(e4, { accounts });

    whitelistAccount = await lockup.account.whitelist(whitelist.publicKey);

    assert.deepEqual(whitelistAccount.entries, [e0, e1, e2, e3, e4]);

    await assert.rejects(
      async () => {
        await lockup.rpc.whitelistAdd(e5, { accounts });
      },
      (err) => {
        assert.equal(err.code, 108);
        assert.equal(err.msg, "Whitelist is full");
        return true;
      }
    );
  });

  it("Removes from the whitelist", async () => {
    await lockup.rpc.whitelistDelete(e0, {
      accounts: {
        authority: provider.wallet.publicKey,
        safe: safe.publicKey,
        whitelist: whitelist.publicKey,
      },
    });
    let whitelistAccount = await lockup.account.whitelist(whitelist.publicKey);
    assert.deepEqual(whitelistAccount.entries, [e1, e2, e3, e4]);
  });

  const vesting = new anchor.web3.Account();
  let vestingAccount = null;
  let vaultAuthority = null;

  it("Creates a vesting account", async () => {
    const beneficiary = provider.wallet.publicKey;
    const endTs = new anchor.BN(Date.now() / 1000 + 3);
    const periodCount = new anchor.BN(5);
    const depositAmount = new anchor.BN(100);

    const vault = new anchor.web3.Account();
    let [
      _vaultAuthority,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [safe.publicKey.toBuffer(), beneficiary.toBuffer()],
      lockup.programId
    );
    vaultAuthority = _vaultAuthority;

    await lockup.rpc.createVesting(
      beneficiary,
      endTs,
      periodCount,
      depositAmount,
      nonce,
      {
        accounts: {
          vesting: vesting.publicKey,
          safe: safe.publicKey,
          vault: vault.publicKey,
          depositor: god,
          depositorAuthority: provider.wallet.publicKey,
          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
        signers: [vesting, vault],
        instructions: [
          await lockup.account.vesting.createInstruction(vesting),
          ...(await serumCmn.createTokenAccountInstrs(
            provider,
            vault.publicKey,
            mint,
            vaultAuthority
          )),
        ],
      }
    );

    vestingAccount = await lockup.account.vesting(vesting.publicKey);

    assert.ok(vestingAccount.safe.equals(safe.publicKey));
    assert.ok(vestingAccount.beneficiary.equals(provider.wallet.publicKey));
    assert.ok(vestingAccount.mint.equals(mint));
    assert.ok(vestingAccount.grantor.equals(provider.wallet.publicKey));
    assert.ok(vestingAccount.outstanding.eq(depositAmount));
    assert.ok(vestingAccount.startBalance.eq(depositAmount));
    assert.ok(vestingAccount.endTs.eq(endTs));
    assert.ok(vestingAccount.periodCount.eq(periodCount));
    assert.ok(vestingAccount.whitelistOwned.eq(new anchor.BN(0)));
    assert.equal(vestingAccount.nonce, nonce);
    assert.ok(endTs.gt(vestingAccount.startTs));
  });

  it("Fails to withdraw from a vesting account before vesting", async () => {
    await assert.rejects(
      async () => {
        await lockup.rpc.withdraw(new anchor.BN(100), {
          accounts: {
            safe: safe.publicKey,
            vesting: vesting.publicKey,
            beneficiary: provider.wallet.publicKey,
            token: god,
            vault: vestingAccount.vault,
            vaultAuthority: vaultAuthority,
            tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          },
        });
      },
      (err) => {
        assert.equal(err.code, 107);
        assert.equal(err.msg, "Insufficient withdrawal balance.");
        return true;
      }
    );
  });

  it("Waits for a vesting period to pass", async () => {
    await serumCmn.sleep(5 * 1000);
  });

  it("Withdraws from the vesting account", async () => {
    const token = await serumCmn.createTokenAccount(
      provider,
      mint,
      provider.wallet.publicKey
    );

    await lockup.rpc.withdraw(new anchor.BN(100), {
      accounts: {
        safe: safe.publicKey,
        vesting: vesting.publicKey,
        beneficiary: provider.wallet.publicKey,
        token,
        vault: vestingAccount.vault,
        vaultAuthority: vaultAuthority,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
    });

    vestingAccount = await lockup.account.vesting(vesting.publicKey);
    assert.ok(vestingAccount.outstanding.eq(new anchor.BN(0)));

    const vaultAccount = await serumCmn.getTokenAccount(
      provider,
      vestingAccount.vault
    );
    assert.ok(vaultAccount.amount.eq(new anchor.BN(0)));

    const tokenAccount = await serumCmn.getTokenAccount(provider, token);
    assert.ok(tokenAccount.amount.eq(new anchor.BN(100)));
  });

  const registrar = new anchor.web3.Account();
  const rewardQ = new anchor.web3.Account();
  const withdrawalTimelock = new anchor.BN(5);
  const maxStake = new anchor.BN("1000000000000000000");
  const stakeRate = new anchor.BN(2);
  const rewardQLen = 100;
  let registrarAccount = null;
  let registrarSigner = null;
  let nonce = null;
  let poolMint = null;

  it("Creates registry genesis", async () => {
    const [
      _registrarSigner,
      _nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer()],
      registry.programId
    );
    registrarSigner = _registrarSigner;
    nonce = _nonce;
    poolMint = await serumCmn.createMint(provider, registrarSigner);
  });

  it("Initializes the registrar", async () => {
    await registry.rpc.initialize(
      mint,
      provider.wallet.publicKey,
      nonce,
      withdrawalTimelock,
      maxStake,
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

    registrarAccount = await registry.account.registrar(registrar.publicKey);

    assert.ok(registrarAccount.authority.equals(provider.wallet.publicKey));
    assert.equal(registrarAccount.nonce, nonce);
    assert.ok(registrarAccount.mint.equals(mint));
    assert.ok(registrarAccount.poolMint.equals(poolMint));
    assert.ok(registrarAccount.stakeRate.eq(stakeRate));
    assert.ok(registrarAccount.rewardEventQ.equals(rewardQ.publicKey));
    assert.ok(registrarAccount.withdrawalTimelock.eq(withdrawalTimelock));
    assert.ok(registrarAccount.maxStake.eq(maxStake));
  });

  const member = new anchor.web3.Account();
  let memberAccount = null;
  let memberSigner = null;
  let balances = null;
  let balancesLocked = null;

  it("Creates a member", async () => {
    const [
      _memberSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer(), member.publicKey.toBuffer()],
      registry.programId
    );
    memberSigner = _memberSigner;

    const [mainTx, _balances] = await utils.createBalanceSandbox(
      provider,
      registrarAccount,
      memberSigner,
      provider.wallet.publicKey // Beneficiary,
    );
    const [lockedTx, _balancesLocked] = await utils.createBalanceSandbox(
      provider,
      registrarAccount,
      memberSigner,
      vesting.publicKey // Lockup.
    );

    balances = _balances;
    balancesLocked = _balancesLocked;

    const tx = registry.transaction.createMember(nonce, {
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

    let txSigs = await provider.sendAll(allTxs);

    memberAccount = await registry.account.member(member.publicKey);

    assert.ok(memberAccount.registrar.equals(registrar.publicKey));
    assert.ok(memberAccount.beneficiary.equals(provider.wallet.publicKey));
    assert.ok(memberAccount.metadata.equals(new anchor.web3.PublicKey()));
    assert.equal(
      JSON.stringify(memberAccount.balances),
      JSON.stringify(balances)
    );
    assert.equal(
      JSON.stringify(memberAccount.balancesLocked),
      JSON.stringify(balancesLocked)
    );
    assert.ok(memberAccount.rewardsCursor === 0);
    assert.ok(memberAccount.lastStakeTs.eq(new anchor.BN(0)));
  });

  it("Deposits (unlocked) to a member", async () => {
    const depositAmount = new anchor.BN(120);
    await registry.rpc.deposit(depositAmount, {
      accounts: {
        // Whitelist relay.
        dummyVesting: anchor.web3.SYSVAR_RENT_PUBKEY,
        depositor: god,
        depositorAuthority: provider.wallet.publicKey,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        vault: memberAccount.balances.vault,
        memberSigner,
        // Program specific.
        registrar: registrar.publicKey,
        beneficiary: provider.wallet.publicKey,
        member: member.publicKey,
      },
    });

    const memberVault = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.vault
    );
    assert.ok(memberVault.amount.eq(depositAmount));
  });

  it("Stakes to a member (unlocked)", async () => {
    const stakeAmount = new anchor.BN(10);
    await registry.rpc.stake(stakeAmount, provider.wallet.publicKey, {
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

  const unlockedVendor = new anchor.web3.Account();
  const unlockedVendorVault = new anchor.web3.Account();
  let unlockedVendorSigner = null;

  it("Drops an unlocked reward", async () => {
    const rewardKind = {
      unlocked: {},
    };
    const rewardAmount = new anchor.BN(200);
    const expiry = new anchor.BN(Date.now() / 1000 + 5);
    const [
      _vendorSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer(), unlockedVendor.publicKey.toBuffer()],
      registry.programId
    );
    unlockedVendorSigner = _vendorSigner;

    await registry.rpc.dropReward(
      rewardKind,
      rewardAmount,
      expiry,
      provider.wallet.publicKey,
      nonce,
      {
        accounts: {
          registrar: registrar.publicKey,
          rewardEventQ: rewardQ.publicKey,
          poolMint,

          vendor: unlockedVendor.publicKey,
          vendorVault: unlockedVendorVault.publicKey,

          depositor: god,
          depositorAuthority: provider.wallet.publicKey,

          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [unlockedVendorVault, unlockedVendor],
        instructions: [
          ...(await serumCmn.createTokenAccountInstrs(
            provider,
            unlockedVendorVault.publicKey,
            mint,
            unlockedVendorSigner
          )),
          await registry.account.rewardVendor.createInstruction(unlockedVendor),
        ],
      }
    );

    const vendorAccount = await registry.account.rewardVendor(
      unlockedVendor.publicKey
    );

    assert.ok(vendorAccount.registrar.equals(registrar.publicKey));
    assert.ok(vendorAccount.vault.equals(unlockedVendorVault.publicKey));
    assert.ok(vendorAccount.nonce === nonce);
    assert.ok(vendorAccount.poolTokenSupply.eq(new anchor.BN(10)));
    assert.ok(vendorAccount.expiryTs.eq(expiry));
    assert.ok(vendorAccount.expiryReceiver.equals(provider.wallet.publicKey));
    assert.ok(vendorAccount.total.eq(rewardAmount));
    assert.ok(vendorAccount.expired === false);
    assert.ok(vendorAccount.rewardEventQCursor === 0);
    assert.deepEqual(vendorAccount.kind, rewardKind);

    const rewardQAccount = await registry.account.rewardQueue(
      rewardQ.publicKey
    );
    assert.ok(rewardQAccount.head === 1);
    assert.ok(rewardQAccount.tail === 0);
    const e = rewardQAccount.events[0];
    assert.ok(e.vendor.equals(unlockedVendor.publicKey));
    assert.equal(e.locked, false);
  });

  it("Collects an unlocked reward", async () => {
    const token = await serumCmn.createTokenAccount(
      provider,
      mint,
      provider.wallet.publicKey
    );
    await registry.rpc.claimRewardUnlocked({
      accounts: {
        token,
        cmn: {
          registrar: registrar.publicKey,

          member: member.publicKey,
          beneficiary: provider.wallet.publicKey,
          balances,
          balancesLocked,

          vendor: unlockedVendor.publicKey,
          vault: unlockedVendorVault.publicKey,
          vendorSigner: unlockedVendorSigner,

          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      },
    });

    let tokenAccount = await serumCmn.getTokenAccount(provider, token);
    assert.ok(tokenAccount.amount.eq(new anchor.BN(200)));

    const memberAccount = await registry.account.member(member.publicKey);
    assert.ok(memberAccount.rewardsCursor == 1);
  });

  const lockedVendor = new anchor.web3.Account();
  const lockedVendorVault = new anchor.web3.Account();
  let lockedVendorSigner = null;
  let lockedRewardAmount = null;
  let lockedRewardKind = null;

  it("Drops a locked reward", async () => {
    lockedRewardKind = {
      locked: {
        endTs: new anchor.BN(Date.now() / 1000 + 70),
        periodCount: new anchor.BN(10),
      },
    };
    lockedRewardAmount = new anchor.BN(200);
    const expiry = new anchor.BN(Date.now() / 1000 + 5);
    const [
      _vendorSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [registrar.publicKey.toBuffer(), lockedVendor.publicKey.toBuffer()],
      registry.programId
    );
    lockedVendorSigner = _vendorSigner;

    await registry.rpc.dropReward(
      lockedRewardKind,
      lockedRewardAmount,
      expiry,
      provider.wallet.publicKey,
      nonce,
      {
        accounts: {
          registrar: registrar.publicKey,
          rewardEventQ: rewardQ.publicKey,
          poolMint,

          vendor: lockedVendor.publicKey,
          vendorVault: lockedVendorVault.publicKey,

          depositor: god,
          depositorAuthority: provider.wallet.publicKey,

          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [lockedVendorVault, lockedVendor],
        instructions: [
          ...(await serumCmn.createTokenAccountInstrs(
            provider,
            lockedVendorVault.publicKey,
            mint,
            lockedVendorSigner
          )),
          await registry.account.rewardVendor.createInstruction(lockedVendor),
        ],
      }
    );

    const vendorAccount = await registry.account.rewardVendor(
      lockedVendor.publicKey
    );

    assert.ok(vendorAccount.registrar.equals(registrar.publicKey));
    assert.ok(vendorAccount.vault.equals(lockedVendorVault.publicKey));
    assert.ok(vendorAccount.nonce === nonce);
    assert.ok(vendorAccount.poolTokenSupply.eq(new anchor.BN(10)));
    assert.ok(vendorAccount.expiryTs.eq(expiry));
    assert.ok(vendorAccount.expiryReceiver.equals(provider.wallet.publicKey));
    assert.ok(vendorAccount.total.eq(lockedRewardAmount));
    assert.ok(vendorAccount.expired === false);
    assert.ok(vendorAccount.rewardEventQCursor === 1);
    assert.equal(
      JSON.stringify(vendorAccount.kind),
      JSON.stringify(lockedRewardKind)
    );

    const rewardQAccount = await registry.account.rewardQueue(
      rewardQ.publicKey
    );
    assert.ok(rewardQAccount.head === 2);
    assert.ok(rewardQAccount.tail === 0);
    const e = rewardQAccount.events[1];
    assert.ok(e.vendor.equals(lockedVendor.publicKey));
    assert.ok(e.locked === true);
  });

  it("Collects a locked reward", async () => {
    const vendoredVesting = new anchor.web3.Account();
    const vendoredVestingVault = new anchor.web3.Account();
    let [
      vendoredVestingSigner,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [safe.publicKey.toBuffer(), provider.wallet.publicKey.toBuffer()],
      lockup.programId
    );
    const remainingAccounts = lockup.instruction.createVesting
      .accounts({
        vesting: vendoredVesting.publicKey,
        safe: safe.publicKey,
        vault: vendoredVestingVault.publicKey,
        depositor: lockedVendorVault.publicKey,
        depositorAuthority: lockedVendorSigner,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      })
      // Change the signer status on the vendor signer since it's signed by the program, not the
      // client.
      .map((meta) =>
        meta.pubkey === lockedVendorSigner ? { ...meta, isSigner: false } : meta
      );

    await registry.rpc.claimRewardLocked(nonce, {
      accounts: {
        lockupProgram: lockup.programId,
        cmn: {
          registrar: registrar.publicKey,

          member: member.publicKey,
          beneficiary: provider.wallet.publicKey,
          balances,
          balancesLocked,

          vendor: lockedVendor.publicKey,
          vault: lockedVendorVault.publicKey,
          vendorSigner: lockedVendorSigner,

          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      },
      remainingAccounts,
      signers: [vendoredVesting, vendoredVestingVault],
      instructions: [
        await lockup.account.vesting.createInstruction(vendoredVesting),
        ...(await serumCmn.createTokenAccountInstrs(
          provider,
          vendoredVestingVault.publicKey,
          mint,
          vendoredVestingSigner
        )),
      ],
    });

    const lockupAccount = await lockup.account.vesting(
      vendoredVesting.publicKey
    );

    assert.ok(lockupAccount.safe.equals(safe.publicKey));
    assert.ok(lockupAccount.beneficiary.equals(provider.wallet.publicKey));
    assert.ok(lockupAccount.mint.equals(mint));
    assert.ok(lockupAccount.vault.equals(vendoredVestingVault.publicKey));
    assert.ok(lockupAccount.outstanding.eq(lockedRewardAmount));
    assert.ok(lockupAccount.startBalance.eq(lockedRewardAmount));
    assert.ok(lockupAccount.endTs.eq(lockedRewardKind.locked.endTs));
    assert.ok(
      lockupAccount.periodCount.eq(lockedRewardKind.locked.periodCount)
    );
    assert.ok(lockupAccount.whitelistOwned.eq(new anchor.BN(0)));
  });

  const pendingWithdrawal = new anchor.web3.Account();

  it("Unstakes", async () => {
    const unstakeAmount = new anchor.BN(10);

    await registry.rpc.startUnstake(unstakeAmount, provider.wallet.publicKey, {
      accounts: {
        registrar: registrar.publicKey,
        rewardEventQ: rewardQ.publicKey,
        poolMint,

        pendingWithdrawal: pendingWithdrawal.publicKey,
        member: member.publicKey,
        beneficiary: provider.wallet.publicKey,
        balances,
        balancesLocked,

        memberSigner,

        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [pendingWithdrawal],
      instructions: [
        await registry.account.pendingWithdrawal.createInstruction(
          pendingWithdrawal
        ),
      ],
    });

    const vaultPw = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.vaultPw
    );
    const vaultStake = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.vaultStake
    );
    const spt = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.spt
    );

    assert.ok(vaultPw.amount.eq(new anchor.BN(20)));
    assert.ok(vaultStake.amount.eq(new anchor.BN(0)));
    assert.ok(spt.amount.eq(new anchor.BN(0)));
  });

  const tryEndUnstake = async () => {
    await registry.rpc.endUnstake({
      accounts: {
        registrar: registrar.publicKey,

        member: member.publicKey,
        beneficiary: provider.wallet.publicKey,
        pendingWithdrawal: pendingWithdrawal.publicKey,

        vault: balances.vault,
        vaultPw: balances.vaultPw,

        memberSigner,

        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });
  };

  it("Fails to end unstaking before timelock", async () => {
    await assert.rejects(
      async () => {
        await tryEndUnstake();
      },
      (err) => {
        assert.equal(err.code, 109);
        assert.equal(err.msg, "The unstake timelock has not yet expired.");
        return true;
      }
    );
  });

  it("Waits for the unstake period to end", async () => {
    await serumCmn.sleep(5000);
  });

  it("Unstake finalizes (unlocked)", async () => {
    await tryEndUnstake();

    const vault = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.vault
    );
    const vaultPw = await serumCmn.getTokenAccount(
      provider,
      memberAccount.balances.vaultPw
    );

    assert.ok(vault.amount.eq(new anchor.BN(120)));
    assert.ok(vaultPw.amount.eq(new anchor.BN(0)));
  });

  it("Withdraws deposits (unlocked)", async () => {
    const token = await serumCmn.createTokenAccount(
      provider,
      mint,
      provider.wallet.publicKey
    );
    const withdrawAmount = new anchor.BN(100);
    await registry.rpc.withdraw(withdrawAmount, {
      accounts: {
        // Whitelist relay.
        dummyVesting: anchor.web3.SYSVAR_RENT_PUBKEY,
        depositor: token,
        depositorAuthority: provider.wallet.publicKey,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        vault: memberAccount.balances.vault,
        memberSigner,
        // Program specific.
        registrar: registrar.publicKey,
        beneficiary: provider.wallet.publicKey,
        member: member.publicKey,
      },
    });

    const tokenAccount = await serumCmn.getTokenAccount(provider, token);
    assert.ok(tokenAccount.amount.eq(withdrawAmount));
  });
});
