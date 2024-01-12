import * as anchor from "@coral-xyz/anchor";

import { TryFrom, IDL } from "../../target/types/try_from";

describe(IDL.name, () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TryFrom as anchor.Program<TryFrom>;

  it("Can use `try_from`", async () => {
    const myAccountKp = anchor.web3.Keypair.generate();
    const FIELD = 5;
    await program.methods
      .init(FIELD)
      .accounts({
        myAccount: myAccountKp.publicKey,
      })
      .signers([myAccountKp])
      .rpc();

    await program.methods
      .tryFrom(FIELD)
      .accounts({
        myAccount: myAccountKp.publicKey,
      })
      .rpc();
  });
});
