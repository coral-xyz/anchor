import * as anchor from "@project-serum/anchor";
import { Program, BN, IdlAccounts, AnchorError } from "@project-serum/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  Token,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Misc } from "../target/types/misc";
import { Misc2 } from "../target/types/misc2";
const utf8 = anchor.utils.bytes.utf8;
const { assert } = require("chai");
const nativeAssert = require("assert");
const miscIdl = require("../target/idl/misc.json");


describe("testing account types", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.Provider.env());
    const program = anchor.workspace.Misc as Program<Misc>;
    const misc2Program = anchor.workspace.Misc2 as Program<Misc2>;

  it("Can use instruction data in accounts constraints", async () => {
    // b"my-seed"
    const seed = Buffer.from([109, 121, 45, 115, 101, 101, 100]);
    const [myPda, nonce] = await PublicKey.findProgramAddress(
      [seed, anchor.web3.SYSVAR_RENT_PUBKEY.toBuffer()],
      program.programId
    );

    await program.rpc.testInstructionConstraint(nonce, {
      accounts: {
        myPda,
        myAccount: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
    });

    it("Can create a PDA account with instruction data", async () => {
      const seed = Buffer.from([1, 2, 3, 4]);
      const domain = "my-domain";
      const foo = anchor.web3.SYSVAR_RENT_PUBKEY;
      const [myPda, nonce] = await PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode("my-seed")),
          Buffer.from(anchor.utils.bytes.utf8.encode(domain)),
          foo.toBuffer(),
          seed,
        ],
        program.programId
      );
  
      await program.rpc.testPdaInit(domain, seed, nonce, {
        accounts: {
          myPda,
          myPayer: program.provider.wallet.publicKey,
          foo,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      });
  
      const myPdaAccount = await program.account.dataU16.fetch(myPda);
      assert.strictEqual(myPdaAccount.data, 6);
    });
  });

  it("Can create a zero copy PDA account", async () => {
    const [myPda, nonce] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-seed"))],
      program.programId
    );
    await program.rpc.testPdaInitZeroCopy({
      accounts: {
        myPda,
        myPayer: program.provider.wallet.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });

    const myPdaAccount = await program.account.dataZeroCopy.fetch(myPda);
    assert.strictEqual(myPdaAccount.data, 9);
    assert.strictEqual(myPdaAccount.bump, nonce);
  });

  it("Can write to a zero copy PDA account", async () => {
    const [myPda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-seed"))],
      program.programId
    );
    await program.rpc.testPdaMutZeroCopy({
      accounts: {
        myPda,
        myPayer: program.provider.wallet.publicKey,
      },
    });

    const myPdaAccount = await program.account.dataZeroCopy.fetch(myPda);
    assert.strictEqual(myPdaAccount.data, 1234);
    assert.strictEqual(myPdaAccount.bump, bump);
  });

  it("Can create a token account from seeds pda", async () => {
    const [mint, mint_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-mint-seed"))],
      program.programId
    );
    const [myPda, token_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed"))],
      program.programId
    );
    await program.rpc.testTokenSeedsInit({
      accounts: {
        myPda,
        mint,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const mintAccount = new Token(
      program.provider.connection,
      mint,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );
    const account = await mintAccount.getAccountInfo(myPda);
    assert.strictEqual(account.state, 1);
    assert.strictEqual(account.amount.toNumber(), 0);
    assert.isTrue(account.isInitialized);
    assert.isTrue(account.owner.equals(program.provider.wallet.publicKey));
    assert.isTrue(account.mint.equals(mint));
  });

  it("Can init a random account", async () => {
    const data = anchor.web3.Keypair.generate();
    await program.rpc.testInit({
      accounts: {
        data: data.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [data],
    });

    const account = await program.account.dataI8.fetch(data.publicKey);
    assert.strictEqual(account.data, 3);
  });

  it("Can init a random account prefunded", async () => {
    const data = anchor.web3.Keypair.generate();
    await program.rpc.testInit({
      accounts: {
        data: data.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [data],
      instructions: [
        anchor.web3.SystemProgram.transfer({
          fromPubkey: program.provider.wallet.publicKey,
          toPubkey: data.publicKey,
          lamports: 4039280,
        }),
      ],
    });

    const account = await program.account.dataI8.fetch(data.publicKey);
    assert.strictEqual(account.data, 3);
  });

  it("Can init a random zero copy account", async () => {
    const data = anchor.web3.Keypair.generate();
    await program.rpc.testInitZeroCopy({
      accounts: {
        data: data.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [data],
    });
    const account = await program.account.dataZeroCopy.fetch(data.publicKey);
    assert.strictEqual(account.data, 10);
    assert.strictEqual(account.bump, 2);
  });

  let mint = undefined;

  it("Can create a random mint account", async () => {
    mint = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint],
    });
    const client = new Token(
      program.provider.connection,
      mint.publicKey,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );
    const mintAccount = await client.getMintInfo();
    assert.strictEqual(mintAccount.decimals, 6);
    assert.isTrue(
      mintAccount.mintAuthority.equals(program.provider.wallet.publicKey)
    );
    assert.isTrue(
      mintAccount.freezeAuthority.equals(program.provider.wallet.publicKey)
    );
  });

  it("Can create a random mint account prefunded", async () => {
    mint = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint],
      instructions: [
        anchor.web3.SystemProgram.transfer({
          fromPubkey: program.provider.wallet.publicKey,
          toPubkey: mint.publicKey,
          lamports: 4039280,
        }),
      ],
    });
    const client = new Token(
      program.provider.connection,
      mint.publicKey,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );
    const mintAccount = await client.getMintInfo();
    assert.strictEqual(mintAccount.decimals, 6);
    assert.isTrue(
      mintAccount.mintAuthority.equals(program.provider.wallet.publicKey)
    );
  });

  it("Can create a random token account", async () => {
    const token = anchor.web3.Keypair.generate();
    await program.rpc.testInitToken({
      accounts: {
        token: token.publicKey,
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [token],
    });
    const client = new Token(
      program.provider.connection,
      mint.publicKey,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );
    const account = await client.getAccountInfo(token.publicKey);
    assert.strictEqual(account.state, 1);
    assert.strictEqual(account.amount.toNumber(), 0);
    assert.isTrue(account.isInitialized);
    assert.isTrue(account.owner.equals(program.provider.wallet.publicKey));
    assert.isTrue(account.mint.equals(mint.publicKey));
  });

  it("Can create a random token with prefunding", async () => {
    const token = anchor.web3.Keypair.generate();
    await program.rpc.testInitToken({
      accounts: {
        token: token.publicKey,
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [token],
      instructions: [
        anchor.web3.SystemProgram.transfer({
          fromPubkey: program.provider.wallet.publicKey,
          toPubkey: token.publicKey,
          lamports: 4039280,
        }),
      ],
    });
    const client = new Token(
      program.provider.connection,
      mint.publicKey,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );
    const account = await client.getAccountInfo(token.publicKey);
    assert.strictEqual(account.state, 1);
    assert.strictEqual(account.amount.toNumber(), 0);
    assert.isTrue(account.isInitialized);
    assert.isTrue(account.owner.equals(program.provider.wallet.publicKey));
    assert.isTrue(account.mint.equals(mint.publicKey));
  });


  it("Can create a random token with prefunding under the rent exemption", async () => {
    const token = anchor.web3.Keypair.generate();
    await program.rpc.testInitToken({
      accounts: {
        token: token.publicKey,
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [token],
      instructions: [
        anchor.web3.SystemProgram.transfer({
          fromPubkey: program.provider.wallet.publicKey,
          toPubkey: token.publicKey,
          lamports: 1,
        }),
      ],
    });
    const client = new Token(
      program.provider.connection,
      mint.publicKey,
      TOKEN_PROGRAM_ID,
      program.provider.wallet.payer
    );
    const account = await client.getAccountInfo(token.publicKey);
    assert.strictEqual(account.state, 1);
    assert.strictEqual(account.amount.toNumber(), 0);
    assert.isTrue(account.isInitialized);
    assert.isTrue(account.owner.equals(program.provider.wallet.publicKey));
    assert.isTrue(account.mint.equals(mint.publicKey));
  });

  describe("associated_token constraints", () => {
    let associatedToken = null;
    // apparently cannot await here so doing it in the 'it' statements
    let client = Token.createMint(
      program.provider.connection,
      program.provider.wallet.payer,
      program.provider.wallet.publicKey,
      program.provider.wallet.publicKey,
      9,
      TOKEN_PROGRAM_ID
    );

    // tests / account-types
    it("Can create an associated token account", async () => {
      const localClient = await client;
      associatedToken = await Token.getAssociatedTokenAddress(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        localClient.publicKey,
        program.provider.wallet.publicKey
      );

      await program.rpc.testInitAssociatedToken({
        accounts: {
          token: associatedToken,
          mint: localClient.publicKey,
          payer: program.provider.wallet.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        },
      });

      const account = await localClient.getAccountInfo(associatedToken);
      assert.strictEqual(account.state, 1);
      assert.strictEqual(account.amount.toNumber(), 0);
      assert.isTrue(account.isInitialized);
      assert.isTrue(account.owner.equals(program.provider.wallet.publicKey));
      assert.isTrue(account.mint.equals(localClient.publicKey));
    });
// tests / account-types
    it("Can validate associated_token constraints", async () => {
      const localClient = await client;
      await program.rpc.testValidateAssociatedToken({
        accounts: {
          token: associatedToken,
          mint: localClient.publicKey,
          wallet: program.provider.wallet.publicKey,
        },
      });

      let otherMint = await Token.createMint(
        program.provider.connection,
        program.provider.wallet.payer,
        program.provider.wallet.publicKey,
        program.provider.wallet.publicKey,
        9,
        TOKEN_PROGRAM_ID
      );

      await nativeAssert.rejects(
        async () => {
          await program.rpc.testValidateAssociatedToken({
            accounts: {
              token: associatedToken,
              mint: otherMint.publicKey,
              wallet: program.provider.wallet.publicKey,
            },
          });
        },
        (err) => {
          assert.strictEqual(err.error.errorCode.number, 2009);
          return true;
        }
      );
    });

    // tests / account-types
    it("associated_token constraints check do not allow authority change", async () => {
      const localClient = await client;
      await program.rpc.testValidateAssociatedToken({
        accounts: {
          token: associatedToken,
          mint: localClient.publicKey,
          wallet: program.provider.wallet.publicKey,
        },
      });

      await localClient.setAuthority(
        associatedToken,
        anchor.web3.Keypair.generate().publicKey,
        "AccountOwner",
        program.provider.wallet.payer,
        []
      );

      await nativeAssert.rejects(
        async () => {
          await program.rpc.testValidateAssociatedToken({
            accounts: {
              token: associatedToken,
              mint: localClient.publicKey,
              wallet: program.provider.wallet.publicKey,
            },
          });
        },
        (err) => {
          assert.strictEqual(err.error.errorCode.number, 2015);
          return true;
        }
      );
    });
  });

  it("init_if_needed throws if account exists but is not owned by the expected program", async () => {
    const newAcc = await anchor.web3.PublicKey.findProgramAddress(
      [utf8.encode("hello")],
      program.programId
    );
    await program.rpc.testInitIfNeededChecksOwner({
      accounts: {
        data: newAcc[0],
        systemProgram: anchor.web3.SystemProgram.programId,
        payer: program.provider.wallet.publicKey,
        owner: program.programId,
      },
    });

    try {
      await program.rpc.testInitIfNeededChecksOwner({
        accounts: {
          data: newAcc[0],
          systemProgram: anchor.web3.SystemProgram.programId,
          payer: program.provider.wallet.publicKey,
          owner: anchor.web3.Keypair.generate().publicKey,
        },
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2004);
    }
  });

  it("init_if_needed throws if pda account exists but does not have the expected seeds", async () => {
    const newAcc = await anchor.web3.PublicKey.findProgramAddress(
      [utf8.encode("nothello")],
      program.programId
    );
    await program.rpc.testInitIfNeededChecksSeeds("nothello", {
      accounts: {
        data: newAcc[0],
        systemProgram: anchor.web3.SystemProgram.programId,
        payer: program.provider.wallet.publicKey,
      },
    });

    // this will throw if it is not a proper PDA
    // we need this so we know that the following tx failed
    // not because it couldn't create this pda
    // but because the two pdas were different
    anchor.web3.PublicKey.createProgramAddress(
      [utf8.encode("hello")],
      program.programId
    );

    try {
      await program.rpc.testInitIfNeededChecksSeeds("hello", {
        accounts: {
          data: newAcc[0],
          systemProgram: anchor.web3.SystemProgram.programId,
          payer: program.provider.wallet.publicKey,
          owner: anchor.web3.Keypair.generate().publicKey,
        },
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2006);
    }
  });

  it("init_if_needed throws if account exists but is not the expected space", async () => {
    const newAcc = anchor.web3.Keypair.generate();
    const _irrelevantForTest = 3;
    await program.rpc.initWithSpace(_irrelevantForTest, {
      accounts: {
        data: newAcc.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        payer: program.provider.wallet.publicKey,
      },
      signers: [newAcc],
    });

    try {
      await program.rpc.testInitIfNeeded(_irrelevantForTest, {
        accounts: {
          data: newAcc.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          payer: program.provider.wallet.publicKey,
        },
        signers: [newAcc],
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2019);
    }
  });

  it("init_if_needed throws if mint exists but has the wrong mint authority", async () => {
    const mint = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint],
    });

    try {
      await program.rpc.testInitMintIfNeeded(6, {
        accounts: {
          mint: mint.publicKey,
          payer: program.provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          mintAuthority: anchor.web3.Keypair.generate().publicKey,
          freezeAuthority: program.provider.wallet.publicKey,
        },
        signers: [mint],
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2016);
    }
  });

  it("init_if_needed throws if mint exists but has the wrong freeze authority", async () => {
    const mint = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint],
    });

    try {
      await program.rpc.testInitMintIfNeeded(6, {
        accounts: {
          mint: mint.publicKey,
          payer: program.provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          mintAuthority: program.provider.wallet.publicKey,
          freezeAuthority: anchor.web3.Keypair.generate().publicKey,
        },
        signers: [mint],
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2017);
    }
  });

  it("init_if_needed throws if mint exists but has the wrong decimals", async () => {
    const mint = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint],
    });

    try {
      await program.rpc.testInitMintIfNeeded(9, {
        accounts: {
          mint: mint.publicKey,
          payer: program.provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          mintAuthority: program.provider.wallet.publicKey,
          freezeAuthority: program.provider.wallet.publicKey,
        },
        signers: [mint],
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2018);
    }
  });

  it("init_if_needed throws if token exists but has the wrong owner", async () => {
    const mint = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint],
    });

    const token = anchor.web3.Keypair.generate();
    await program.rpc.testInitToken({
      accounts: {
        token: token.publicKey,
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [token],
    });

    try {
      await program.rpc.testInitTokenIfNeeded({
        accounts: {
          token: token.publicKey,
          mint: mint.publicKey,
          payer: program.provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          authority: anchor.web3.Keypair.generate().publicKey,
        },
        signers: [token],
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2015);
    }
  });

  it("init_if_needed throws if token exists but has the wrong mint", async () => {
    const mint = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint],
    });

    const mint2 = anchor.web3.Keypair.generate();
    await program.rpc.testInitMint({
      accounts: {
        mint: mint2.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [mint2],
    });

    const token = anchor.web3.Keypair.generate();
    await program.rpc.testInitToken({
      accounts: {
        token: token.publicKey,
        mint: mint.publicKey,
        payer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [token],
    });

    try {
      await program.rpc.testInitTokenIfNeeded({
        accounts: {
          token: token.publicKey,
          mint: mint2.publicKey,
          payer: program.provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          authority: program.provider.wallet.publicKey,
        },
        signers: [token],
      });
      assert.ok(false);
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      assert.strictEqual(err.error.errorCode.number, 2014);
    }
  });

   



});

