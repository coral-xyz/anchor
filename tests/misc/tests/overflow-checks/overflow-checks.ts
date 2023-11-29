import * as anchor from "@coral-xyz/anchor";

import { OverflowChecks, IDL } from "../../target/types/overflow_checks";

describe(IDL.name, () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .OverflowChecks as anchor.Program<OverflowChecks>;

  const accountKp = anchor.web3.Keypair.generate();
  const testOverflow = async (method: keyof typeof program["methods"]) => {
    try {
      await program.methods[method]()
        .accounts({
          account: accountKp.publicKey,
        })
        .rpc();
    } catch (e) {
      if (e.logs.some((log) => log.includes("with overflow"))) return;
      throw e;
    }

    throw new Error("Did not panic on overflow");
  };

  before(async () => {
    await program.methods
      .initialize()
      .accounts({
        account: accountKp.publicKey,
        payer: program.provider.publicKey,
      })
      .signers([accountKp])
      .rpc();
  });

  it("Panics on overflow add", async () => {
    await testOverflow("testOverflowAdd");
  });

  it("Panics on overflow sub", async () => {
    await testOverflow("testOverflowSub");
  });

  it("Panics on overflow mul", async () => {
    await testOverflow("testOverflowMul");
  });
});
