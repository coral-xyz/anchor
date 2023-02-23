import TransactionInstructionsFactory from "../src/program/namespace/transaction-instructions";
import InstructionFactory from "../src/program/namespace/instruction";
import { BorshCoder } from "../src";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

describe("Transaction Instructions", () => {
  const preIx = new TransactionInstruction({
    keys: [],
    programId: PublicKey.default,
    data: Buffer.from("pre"),
  });
  const postIx = new TransactionInstruction({
    keys: [],
    programId: PublicKey.default,
    data: Buffer.from("post"),
  });
  const idl = {
    version: "0.0.0",
    name: "basic_0",
    instructions: [
      {
        name: "initialize",
        accounts: [],
        args: [],
      },
    ],
  };

  it("should add pre instructions before method ix", async () => {
    const coder = new BorshCoder(idl);
    const programId = PublicKey.default;
    const ixItem = InstructionFactory.build(
      idl.instructions[0],
      (ixName, ix) => coder.instruction.encode(ixName, ix),
      programId
    );
    const txIxsItem = TransactionInstructionsFactory.build(
      idl.instructions[0],
      ixItem
    );
    const txIxs = txIxsItem({ accounts: {}, preInstructions: [preIx] });
    expect(txIxs.length).toBe(2);
    expect(txIxs[0]).toMatchObject(preIx);
  });

  it("should add post instructions after method ix", async () => {
    const coder = new BorshCoder(idl);
    const programId = PublicKey.default;
    const ixItem = InstructionFactory.build(
      idl.instructions[0],
      (ixName, ix) => coder.instruction.encode(ixName, ix),
      programId
    );
    const txIxsItem = TransactionInstructionsFactory.build(
      idl.instructions[0],
      ixItem
    );
    const txIxs = txIxsItem({ accounts: {}, postInstructions: [postIx] });
    expect(txIxs.length).toBe(2);
    expect(txIxs[1]).toMatchObject(postIx);
  });

  it("should throw error if both preInstructions and instructions are used", async () => {
    const coder = new BorshCoder(idl);
    const programId = PublicKey.default;
    const ixItem = InstructionFactory.build(
      idl.instructions[0],
      (ixName, ix) => coder.instruction.encode(ixName, ix),
      programId
    );
    const txIxsItem = TransactionInstructionsFactory.build(
      idl.instructions[0],
      ixItem
    );

    expect(() =>
      txIxsItem({
        accounts: {},
        preInstructions: [preIx],
        instructions: [preIx],
      })
    ).toThrow(new Error("instructions is deprecated, use preInstructions"));
  });
});
