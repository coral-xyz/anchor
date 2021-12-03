import TransactionFactory from "../src/program/namespace/transaction";
import InstructionFactory from "../src/program/namespace/instruction";
import { Coder } from "../src";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

describe("Transaction", () => {
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
    const coder = new Coder(idl);
    const programId = PublicKey.default;
    const ixItem = InstructionFactory.build(
      idl.instructions[0],
      (ixName, ix) => coder.instruction.encode(ixName, ix),
      programId
    );
    const txItem = TransactionFactory.build(idl.instructions[0], ixItem);
    const tx = txItem({ accounts: {}, preInstructions: [preIx] });
    expect(tx.instructions.length).toBe(2);
    expect(tx.instructions[0]).toMatchObject(preIx);
  });

  it("should add post instructions after method ix", async () => {
    const coder = new Coder(idl);
    const programId = PublicKey.default;
    const ixItem = InstructionFactory.build(
      idl.instructions[0],
      (ixName, ix) => coder.instruction.encode(ixName, ix),
      programId
    );
    const txItem = TransactionFactory.build(idl.instructions[0], ixItem);
    const tx = txItem({ accounts: {}, postInstructions: [postIx] });
    expect(tx.instructions.length).toBe(2);
    expect(tx.instructions[1]).toMatchObject(postIx);
  });

  it("should throw error if both preInstructions and instructions are used", async () => {
    const coder = new Coder(idl);
    const programId = PublicKey.default;
    const ixItem = InstructionFactory.build(
      idl.instructions[0],
      (ixName, ix) => coder.instruction.encode(ixName, ix),
      programId
    );
    const txItem = TransactionFactory.build(idl.instructions[0], ixItem);

    expect(() =>
      txItem({ accounts: {}, preInstructions: [preIx], instructions: [preIx] })
    ).toThrow(new Error("instructions is deprecated, use preInstructions"));
  });
});
