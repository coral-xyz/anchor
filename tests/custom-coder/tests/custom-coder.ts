//import * as anchor from '@project-serum/anchor';
//import { Program } from '@project-serum/anchor';
import * as assert from "assert";
import { SystemProgram, Keypair, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as anchor from "../../../ts";
import { Spl } from "../../../ts";

describe("custom-coder", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  // Client.
  const program = Spl.token();

  // Constants.
  const mintKeypair = Keypair.generate();
  const tokenKeypair = Keypair.generate();
  const rent = SYSVAR_RENT_PUBKEY;

  it("Creates a mint", async () => {
    await program.rpc.initializeMint(
      6,
      program.provider.wallet.publicKey,
      null,
      {
        accounts: {
          mint: mintKeypair.publicKey,
          rent,
        },
        signers: [mintKeypair],
        preInstructions: [
          SystemProgram.createAccount({
            fromPubkey: program.provider.wallet.publicKey,
            newAccountPubkey: mintKeypair.publicKey,
            lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
              82
            ),
            space: 82,
            programId: TOKEN_PROGRAM_ID,
          }),
        ],
      }
    );
    const mintAccount = await program.account.mint.fetch(mintKeypair.publicKey);
    assert.ok(
      mintAccount.mintAuthority.equals(program.provider.wallet.publicKey)
    );
    console.log(mintAccount);
  });

  it("Creates a token account", async () => {
    await program.rpc.initializeAccount({
      accounts: {
        account: tokenKeypair.publicKey,
        mint: mintKeypair.publicKey,
        authority: program.provider.wallet.publicKey,
        rent,
      },
      signers: [tokenKeypair],
      preInstructions: [
        SystemProgram.createAccount({
          fromPubkey: program.provider.wallet.publicKey,
          newAccountPubkey: tokenKeypair.publicKey,
          lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
            165
          ),
          space: 165,
          programId: TOKEN_PROGRAM_ID,
        }),
      ],
    });
    const token = await program.account.token.fetch(tokenKeypair.publicKey);
    assert.ok(token.authority.equals(program.provider.wallet.publicKey));
    assert.ok(token.mint.equals(mintKeypair.publicKey));
    console.log("token", token);
  });
});
