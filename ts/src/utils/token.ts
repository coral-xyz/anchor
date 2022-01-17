import { PublicKey, TransactionInstruction, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";

export const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

export async function associatedAddress({
  mint,
  owner,
}: {
  mint: PublicKey;
  owner: PublicKey;
}): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )
  )[0];
}

/**
 * prepares the instruction to create an associated-token-account
 * @param associatedAddress: ATA address, obtain with await associatedAddress(...)
 * @param payer: fee-payer, must be tx signer later
 * @returns TransactionInstruction
 */
 export function createAssociatedTokenAccountInstruction(
  associatedAddress: PublicKey,
  {
    mint,
    owner,
  }: {
    mint: PublicKey;
    owner: PublicKey;
  },
  payer: PublicKey): TransactionInstruction {
  const data = Buffer.alloc(0);
  let keys = [{
      pubkey: payer,
      isSigner: true,
      isWritable: true
  }, {
      pubkey: associatedAddress,
      isSigner: false,
      isWritable: true
  }, {
      pubkey: owner,
      isSigner: false,
      isWritable: false
  }, {
      pubkey: mint,
      isSigner: false,
      isWritable: false
  }, {
      pubkey: SystemProgram.programId,
      isSigner: false,
      isWritable: false
  }, {
      pubkey: TOKEN_PROGRAM_ID,
      isSigner: false,
      isWritable: false
  }, {
      pubkey: SYSVAR_RENT_PUBKEY,
      isSigner: false,
      isWritable: false
  }];
  return new TransactionInstruction({
      keys,
      programId: ASSOCIATED_TOKEN_PROGRAM_ID,
      data
  });
}
