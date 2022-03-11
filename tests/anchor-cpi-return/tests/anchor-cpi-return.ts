import assert from "assert";
import * as anchor from "@project-serum/anchor";
import * as borsh from "borsh";
import { Program } from "@project-serum/anchor";
import { AnchorCpiReturn } from "../target/types/anchor_cpi_return";
import { AnchorCpiCaller } from "../target/types/anchor_cpi_caller";

const { SystemProgram } = anchor.web3;

describe("anchor-cpi-return", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const callerProgram = anchor.workspace
    .AnchorCpiCaller as Program<AnchorCpiCaller>;
  const returnProgram = anchor.workspace
    .AnchorCpiReturn as Program<AnchorCpiReturn>;

  const getReturnLog = (confirmedTransaction) => {
    const prefix = "Program return: ";
    let log = confirmedTransaction.meta.logMessages.find((log) =>
      log.startsWith(prefix)
    );
    log = log.slice(prefix.length);
    const [key, data] = log.split(" ", 2);
    const buffer = Buffer.from(data, "base64");
    return [key, data, buffer];
  };

  const cpiReturn = anchor.web3.Keypair.generate();

  const confirmOptions = { commitment: "confirmed" };

  it("can initialize", async () => {
    await returnProgram.methods
      .initialize()
      .accounts({
        account: cpiReturn.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([cpiReturn])
      .rpc();
  });

  it("can return u64 from a cpi", async () => {
    const tx = await callerProgram.methods
      .cpiCallReturnU64()
      .accounts({
        cpiReturn: cpiReturn.publicKey,
        cpiReturnProgram: returnProgram.programId,
      })
      .rpc(confirmOptions);
    let t = await provider.connection.getTransaction(tx, {
      commitment: "confirmed",
    });

    const [key, data, buffer] = getReturnLog(t);
    assert.equal(key, returnProgram.programId);

    // Check for matching log on receive side
    let receiveLog = t.meta.logMessages.find(
      (log) => log == `Program log: ${data}`
    );
    assert(receiveLog !== undefined);

    const reader = new borsh.BinaryReader(buffer);
    assert.equal(reader.readU64().toNumber(), 10);
  });

  it("can make a non-cpi call to a function that returns a u64", async () => {
    const tx = await returnProgram.methods
      .returnU64()
      .accounts({
        account: cpiReturn.publicKey,
      })
      .rpc(confirmOptions);
    let t = await provider.connection.getTransaction(tx, {
      commitment: "confirmed",
    });
    const [key, , buffer] = getReturnLog(t);
    assert.equal(key, returnProgram.programId);
    const reader = new borsh.BinaryReader(buffer);
    assert.equal(reader.readU64().toNumber(), 10);
  });

  it("can return a struct from a cpi", async () => {
    const tx = await callerProgram.methods
      .cpiCallReturnStruct()
      .accounts({
        cpiReturn: cpiReturn.publicKey,
        cpiReturnProgram: returnProgram.programId,
      })
      .rpc(confirmOptions);
    let t = await provider.connection.getTransaction(tx, {
      commitment: "confirmed",
    });

    const [key, data, buffer] = getReturnLog(t);
    assert.equal(key, returnProgram.programId);

    // Check for matching log on receive side
    let receiveLog = t.meta.logMessages.find(
      (log) => log == `Program log: ${data}`
    );
    assert(receiveLog !== undefined);

    // Deserialize the struct and validate
    class Assignable {
      constructor(properties) {
        Object.keys(properties).map((key) => {
          this[key] = properties[key];
        });
      }
    }
    class Data extends Assignable {}
    const schema = new Map([
      [Data, { kind: "struct", fields: [["value", "u64"]] }],
    ]);
    const deserialized = borsh.deserialize(schema, Data, buffer);
    assert(deserialized.value.toNumber() === 11);
  });
});
