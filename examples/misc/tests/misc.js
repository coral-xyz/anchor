const anchor = require("@project-serum/anchor");
const serumCmn = require("@project-serum/common");
const assert = require("assert");

describe("misc", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Misc;
  const misc2Program = anchor.workspace.Misc2;

  it("Can allocate extra space for a state constructor", async () => {
    const tx = await program.state.rpc.new();
    const addr = await program.state.address();
    const state = await program.state.fetch();
    const accountInfo = await program.provider.connection.getAccountInfo(addr);
    assert.ok(state.v.equals(Buffer.from([])));
    assert.ok(accountInfo.data.length === 99);
  });

  const data = anchor.web3.Keypair.generate();

  it("Can use u128 and i128", async () => {
    const tx = await program.rpc.initialize(
      new anchor.BN(1234),
      new anchor.BN(22),
      {
        accounts: {
          data: data.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [data],
        instructions: [await program.account.data.createInstruction(data)],
      }
    );
    const dataAccount = await program.account.data.fetch(data.publicKey);
    assert.ok(dataAccount.udata.eq(new anchor.BN(1234)));
    assert.ok(dataAccount.idata.eq(new anchor.BN(22)));
  });

  it("Can use u16", async () => {
    const data = anchor.web3.Keypair.generate();
    const tx = await program.rpc.testU16(99, {
      accounts: {
        myAccount: data.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [data],
      instructions: [await program.account.dataU16.createInstruction(data)],
    });
    const dataAccount = await program.account.dataU16.fetch(data.publicKey);
    assert.ok(dataAccount.data === 99);
  });

  it("Can embed programs into genesis from the Anchor.toml", async () => {
    const pid = new anchor.web3.PublicKey(
      "FtMNMKp9DZHKWUyVAsj3Q5QV8ow4P3fUPP7ZrWEQJzKr"
    );
    let accInfo = await anchor.getProvider().connection.getAccountInfo(pid);
    assert.ok(accInfo.executable);
  });

  it("Can use the owner constraint", async () => {
    await program.rpc.testOwner({
      accounts: {
        data: data.publicKey,
        misc: program.programId,
      },
    });

    await assert.rejects(
      async () => {
        await program.rpc.testOwner({
          accounts: {
            data: program.provider.wallet.publicKey,
            misc: program.programId,
          },
        });
      },
      (err) => {
        return true;
      }
    );
  });

  it("Can use the executable attribute", async () => {
    await program.rpc.testExecutable({
      accounts: {
        program: program.programId,
      },
    });

    await assert.rejects(
      async () => {
        await program.rpc.testExecutable({
          accounts: {
            program: program.provider.wallet.publicKey,
          },
        });
      },
      (err) => {
        return true;
      }
    );
  });

  it("Can CPI to state instructions", async () => {
    const oldData = new anchor.BN(0);
    await misc2Program.state.rpc.new({
      accounts: {
        authority: program.provider.wallet.publicKey,
      },
    });
    let stateAccount = await misc2Program.state.fetch();
    assert.ok(stateAccount.data.eq(oldData));
    assert.ok(stateAccount.auth.equals(program.provider.wallet.publicKey));
    const newData = new anchor.BN(2134);
    await program.rpc.testStateCpi(newData, {
      accounts: {
        authority: program.provider.wallet.publicKey,
        cpiState: await misc2Program.state.address(),
        misc2Program: misc2Program.programId,
      },
    });
    stateAccount = await misc2Program.state.fetch();
    assert.ok(stateAccount.data.eq(newData));
    assert.ok(stateAccount.auth.equals(program.provider.wallet.publicKey));
  });

  it("Can init an associated program account", async () => {
    const state = await program.state.address();

    // Manual associated address calculation for test only. Clients should use
    // the generated methods.
    const [
      associatedAccount,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from([97, 110, 99, 104, 111, 114]), // b"anchor".
        program.provider.wallet.publicKey.toBuffer(),
        state.toBuffer(),
        data.publicKey.toBuffer(),
      ],
      program.programId
    );
    await assert.rejects(
      async () => {
        await program.account.testData.fetch(associatedAccount);
      },
      (err) => {
        assert.ok(
          err.toString() ===
            `Error: Account does not exist ${associatedAccount.toString()}`
        );
        return true;
      }
    );
    await program.rpc.testInitAssociatedAccount(new anchor.BN(1234), {
      accounts: {
        myAccount: associatedAccount,
        authority: program.provider.wallet.publicKey,
        state,
        data: data.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    // Try out the generated associated method.
    const account = await program.account.testData.associated(
      program.provider.wallet.publicKey,
      state,
      data.publicKey
    );
    assert.ok(account.data.toNumber() === 1234);
  });

  it("Can use an associated program account", async () => {
    const state = await program.state.address();
    const [
      associatedAccount,
      nonce,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from([97, 110, 99, 104, 111, 114]), // b"anchor".
        program.provider.wallet.publicKey.toBuffer(),
        state.toBuffer(),
        data.publicKey.toBuffer(),
      ],
      program.programId
    );
    await program.rpc.testAssociatedAccount(new anchor.BN(5), {
      accounts: {
        myAccount: associatedAccount,
        authority: program.provider.wallet.publicKey,
        state,
        data: data.publicKey,
      },
    });
    // Try out the generated associated method.
    const account = await program.account.testData.associated(
      program.provider.wallet.publicKey,
      state,
      data.publicKey
    );
    assert.ok(account.data.toNumber() === 5);
  });

  it("Can retrieve events when simulating a transaction", async () => {
    const resp = await program.simulate.testSimulate(44);
    const expectedRaw = [
      "Program Z2Ddx1Lcd8CHTV9tkWtNnFQrSz6kxz2H38wrr18zZRZ invoke [1]",
      "Program log: NgyCA9omwbMsAAAA",
      "Program log: fPhuIELK/k7SBAAA",
      "Program log: jvbowsvlmkcJAAAA",
      "Program Z2Ddx1Lcd8CHTV9tkWtNnFQrSz6kxz2H38wrr18zZRZ consumed 4819 of 200000 compute units",
      "Program Z2Ddx1Lcd8CHTV9tkWtNnFQrSz6kxz2H38wrr18zZRZ success",
    ];
    assert.ok(JSON.stringify(expectedRaw), resp.raw);
    assert.ok(resp.events[0].name === "E1");
    assert.ok(resp.events[0].data.data === 44);
    assert.ok(resp.events[1].name === "E2");
    assert.ok(resp.events[1].data.data === 1234);
    assert.ok(resp.events[2].name === "E3");
    assert.ok(resp.events[2].data.data === 9);
  });

  it("Can use i8 in the idl", async () => {
    const data = anchor.web3.Keypair.generate();
    await program.rpc.testI8(-3, {
      accounts: {
        data: data.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      instructions: [await program.account.dataI8.createInstruction(data)],
      signers: [data],
    });
    const dataAccount = await program.account.dataI8.fetch(data.publicKey);
    assert.ok(dataAccount.data === -3);
  });

  let dataPubkey;

  it("Can use i16 in the idl", async () => {
    const data = anchor.web3.Keypair.generate();
    await program.rpc.testI16(-2048, {
      accounts: {
        data: data.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      instructions: [await program.account.dataI16.createInstruction(data)],
      signers: [data],
    });
    const dataAccount = await program.account.dataI16.fetch(data.publicKey);
    assert.ok(dataAccount.data === -2048);

    dataPubkey = data.publicKey;
  });

  it("Can use base58 strings to fetch an account", async () => {
    const dataAccount = await program.account.dataI16.fetch(
      dataPubkey.toString()
    );
    assert.ok(dataAccount.data === -2048);
  });

  it("Should fail to close an account when sending lamports to itself", async () => {
    try {
      await program.rpc.testClose({
        accounts: {
          data: data.publicKey,
          solDest: data.publicKey,
        },
      });
      assert.ok(false);
    } catch (err) {
      const errMsg = "A close constraint was violated";
      assert.equal(err.toString(), errMsg);
      assert.equal(err.msg, errMsg);
      assert.equal(err.code, 151);
    }
  });

  it("Can close an account", async () => {
    const openAccount = await program.provider.connection.getAccountInfo(
      data.publicKey
    );
    assert.ok(openAccount !== null);

    let beforeBalance = (
      await program.provider.connection.getAccountInfo(
        program.provider.wallet.publicKey
      )
    ).lamports;

    await program.rpc.testClose({
      accounts: {
        data: data.publicKey,
        solDest: program.provider.wallet.publicKey,
      },
    });

    let afterBalance = (
      await program.provider.connection.getAccountInfo(
        program.provider.wallet.publicKey
      )
    ).lamports;

    // Retrieved rent exemption sol.
    assert.ok(afterBalance > beforeBalance);

    const closedAccount = await program.provider.connection.getAccountInfo(
      data.publicKey
    );
    assert.ok(closedAccount === null);
  });
});
