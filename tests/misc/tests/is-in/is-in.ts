import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import { IsIn } from "../../target/types/is_in";

describe("is-in", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.IsIn as Program<IsIn>;

  const mint = new web3.PublicKey(
    "BEcGFQK1T1tSu3kvHC17cyCkQ5dvXqAJ7ExB2bb5Do7a"
  );
  const vault = new web3.PublicKey(
    "FSRvxBNrQWX2Fy2qvKMLL3ryEdRtE3PUTZBcdKwASZTU"
  );
  const newMint = new web3.PublicKey(
    "57z5KG1EHj5SV79xR1GVzEvkjWSJHgA7XMuPE457Rain"
  );
  const config = new web3.PublicKey(
    "14Wp3dxYTQpRMMz3AW7f2XGBTdaBrf1qb2NKjAN3Tb13"
  );

  let market: web3.PublicKey;
  let zeroMarket: web3.PublicKey;
  const authoritySig = web3.Keypair.generate();
  const authority = authoritySig.publicKey;

  it("Is initialized!", async () => {
    [market] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("MARKET")],
      program.programId
    );
    [zeroMarket] = await web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("ZERO_MARKET")],
      program.programId
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(authority, 10_000_000_000),
      "confirmed"
    );

    await program.methods
      .initialize(mint, vault, config)
      .accounts({
        market,
        zeroMarket,
        authority,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([authoritySig])
      .rpc();
  });

  it("Is updated!", async () => {
    await program.methods
      .update(newMint, vault, config)
      .accounts({
        market,
        zeroMarket,
        authority,
      })
      .signers([authoritySig])
      .rpc();
  });

  it("Is not closed with old mint!", async () => {
    try {
      await program.methods
        .close()
        .accounts({
          market,
          zeroMarket,
          authority,
          mint,
          vault,
          config,
        })
        .signers([authoritySig])
        .rpc();
    } catch {
      return;
    }
    throw "expected to fail, but tx was fullfilled";
  });

  it("Is closed!", async () => {
    await program.methods
      .close()
      .accounts({
        market,
        zeroMarket,
        authority,
        mint: newMint,
        vault,
        config,
      })
      .signers([authoritySig])
      .rpc();
  });
});
