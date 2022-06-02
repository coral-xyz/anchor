import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { Realloc } from "../target/types/realloc";

describe("realloc", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Realloc as Program<Realloc>;
  const authority = (program.provider as any).wallet
    .payer as anchor.web3.Keypair;

  const [sample] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("sample")],
    program.programId
  );

  let postAllocBalance: number;

  it("Is initialized!", async () => {
    const b = await program.provider.connection.getBalance(authority.publicKey);
    console.log(b);

    await program.methods
      .initialize()
      .accounts({ authority: authority.publicKey, sample })
      .rpc();

    const s = await program.account.sample.fetch(sample);
    assert.lengthOf(s.data, 1);

    postAllocBalance = await program.provider.connection.getBalance(
      authority.publicKey
    );
    console.log(postAllocBalance);
  });

  it("realloc additive", async () => {
    await program.methods
      .realloc(5)
      .accounts({ authority: authority.publicKey, sample })
      .rpc();

    const s = await program.account.sample.fetch(sample);
    assert.lengthOf(s.data, 5);

    const b = await program.provider.connection.getBalance(authority.publicKey);
    console.log(b);
  });

  it("realloc substractive", async () => {
    await program.methods
      .realloc(1)
      .accounts({ authority: authority.publicKey, sample })
      .rpc();

    const s = await program.account.sample.fetch(sample);
    assert.lengthOf(s.data, 1);

    const b = await program.provider.connection.getBalance(authority.publicKey);
    console.log(b);
    assert.strictEqual(b, postAllocBalance);
  });
});
