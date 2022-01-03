import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMintInfo, getTokenAccount } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
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
    const vault = await findAssociatedTokenAddress(program.provider.wallet.publicKey, mint.publicKey);

    await program.rpc.createMintAndVault(decimals, new anchor.BN(amount), {
      accounts: {
        authority: program.provider.wallet.publicKey,
        mint: mint.publicKey,
        vault: vault,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [mint],
    });

    const mintInfo = await getMintInfo(program.provider, mint.publicKey);
    assert.equal(mintInfo.decimals, decimals);
    assert.equal(mintInfo.supply.toNumber(), amount);

    const vaultAccountInfo = await getTokenAccount(program.provider, vault);
    assert.equal(vaultAccountInfo.amount.toNumber(), amount);
    assert.deepEqual(vaultAccountInfo.mint, mint.publicKey);
    assert.deepEqual(vaultAccountInfo.owner, program.provider.wallet.publicKey);
  });
});

async function findAssociatedTokenAddress(walletAddress: anchor.web3.PublicKey, tokenMintAddress: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> {
  return (await anchor.web3.PublicKey.findProgramAddress([walletAddress.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), tokenMintAddress.toBuffer()], ASSOCIATED_TOKEN_PROGRAM_ID))[0];
}