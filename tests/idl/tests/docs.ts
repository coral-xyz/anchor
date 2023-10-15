import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";

import { Docs } from "../target/types/docs";

describe("Docs", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.docs as Program<Docs>;

  const instruction = program.idl.instructions.find(
    (i) => i.name === "testIdlDocParse"
  );

  it("includes instruction doc comment", () => {
    assert.deepEqual(instruction.docs, [
      "This instruction doc should appear in the IDL",
    ]);
  });

  it("includes account doc comment", () => {
    const act = instruction.accounts.find((i) => i.name === "act");
    assert.deepEqual(act.docs, [
      "This account doc comment should appear in the IDL",
      "This is a multi-line comment",
    ]);
  });

  const dataWithDoc = program.idl.accounts.find(
    // @ts-expect-error
    (acc) => acc.name === "DataWithDoc"
  );

  it("includes accounts doc comment", () => {
    assert.deepEqual(dataWithDoc.docs, [
      "Custom account doc comment should appear in the IDL",
    ]);
  });

  it("includes account attribute doc comment", () => {
    const dataField = dataWithDoc.type.fields.find((i) => i.name === "data");
    assert.deepEqual(dataField.docs, [
      "Account attribute doc comment should appear in the IDL",
    ]);
  });
});
