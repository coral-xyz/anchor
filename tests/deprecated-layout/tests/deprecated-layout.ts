import * as assert from "assert";
import * as anchor from "@project-serum/anchor";
import { Program, BorshAccountHeader } from "@project-serum/anchor";
import { Keypair } from "@solana/web3.js";
import { DeprecatedLayout } from "../target/types/deprecated_layout";
import { NewLayout } from "../target/types/new_layout";

describe("deprecated-layout", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  it("Has an 8 byte discriminator", async () => {
    const program = anchor.workspace
      .DeprecatedLayout as Program<DeprecatedLayout>;

    const dataKeypair = Keypair.generate();
    await program.methods
      .initialize()
      .accounts({ data: dataKeypair.publicKey })
      .signers([dataKeypair])
      .rpc();
    const accountInfo = await program.account.data.getAccountInfo(
      dataKeypair.publicKey
    );
    const data = accountInfo.data;
    const header = data.slice(0, 8);
    const accountData = data.slice(8);
    const expectedDiscriminator = new BorshAccountHeader(
      program.idl
    ).discriminator("data");

    assert.ok(
      "0xce9c3bbc124ff0e8" ===
        anchor.utils.bytes.hex.encode(expectedDiscriminator)
    );
    assert.ok(expectedDiscriminator.length === 8);
    assert.ok(header.compare(expectedDiscriminator) === 0);
    assert.ok(accountData.compare(Buffer.from([2, 0, 0, 0, 0, 0, 0, 0])) === 0);

    const dataAccount = await program.account.data.fetch(dataKeypair.publicKey);
    assert.ok(dataAccount.data.toNumber() === 2);
  });

  it("Has a 4 byte discriminator and 8 byte header", async () => {
    const program = anchor.workspace.NewLayout as Program<NewLayout>;

    const dataKeypair = Keypair.generate();
    await program.methods
      .initialize()
      .accounts({ data: dataKeypair.publicKey })
      .signers([dataKeypair])
      .rpc();
    const accountInfo = await program.account.data.getAccountInfo(
      dataKeypair.publicKey
    );
    const data = accountInfo.data;
    const header = data.slice(0, 8);
    const givenDiscriminator = header.slice(2, 6);
    const accountData = data.slice(8);
    const expectedDiscriminator = new BorshAccountHeader(
      program.idl
    ).discriminator("data");

    assert.ok(
      "0xce9c3bbc" === anchor.utils.bytes.hex.encode(expectedDiscriminator)
    );
    assert.ok(expectedDiscriminator.length === 4);
    assert.ok(givenDiscriminator.compare(expectedDiscriminator) === 0);
    assert.ok(accountData.compare(Buffer.from([2, 0, 0, 0, 0, 0, 0, 0])) === 0);
    assert.ok(header[0] === 1);
    assert.ok(header[1] === 1);
    assert.ok(header[6] === 0);
    assert.ok(header[7] === 0);

    const dataAccount = await program.account.data.fetch(dataKeypair.publicKey);
    assert.ok(dataAccount.data.toNumber() === 2);
  });
});
