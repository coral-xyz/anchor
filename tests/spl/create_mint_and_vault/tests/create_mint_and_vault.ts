import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMintInfo, getTokenAccount } from "@project-serum/common";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { CreateMintAndVault } from "../target/types/create_mint_and_vault";
import assert from "assert";

describe("create_mint_and_vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.CreateMintAndVault as Program<CreateMintAndVault>;

  it("create_mint_and_vault", async () => {
    const decimals = 0;
    const amount = 1000;

    const mint = anchor.web3.Keypair.generate();
    const vault = anchor.web3.Keypair.generate();

    await program.rpc.createMintAndVault(decimals, new anchor.BN(amount), {
      accounts: {
        authority: program.provider.wallet.publicKey,
        mint: mint.publicKey,
        vault: vault.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [mint, vault],
    });
    // console.log("tx", tx);

    const mintInfo = await getMintInfo(program.provider, mint.publicKey);
    assert.equal(mintInfo.decimals, decimals);
    assert.equal(mintInfo.supply.toNumber(), amount);

    const vaultAccountInfo = await getTokenAccount(program.provider, vault.publicKey);
    assert.equal(vaultAccountInfo.amount.toNumber(), amount);
    assert.deepEqual(vaultAccountInfo.mint, mint.publicKey);
    assert.deepEqual(vaultAccountInfo.owner, program.provider.wallet.publicKey);
  });
});
