import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ValidatorWarpSlot } from "../target/types/validator_warp_slot";

describe("validator-warp-slot", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace
    .ValidatorWarpSlot as Program<ValidatorWarpSlot>;

  it("Has the same slot as mainnet", async () => {
    const mainnetConnection = new anchor.web3.Connection(
      "https://api.mainnet-beta.solana.com"
    );

    const [clockInfo] = await anchor.utils.rpc.getMultipleAccounts(
      mainnetConnection,
      [anchor.web3.SYSVAR_CLOCK_PUBKEY]
    );

    if (!clockInfo) {
      throw new Error("Unable to fetch clock account from mainnet");
    }

    const mainnetSlot = new anchor.BN(clockInfo.account.data.slice(0, 8), "le");
    const tx = await program.methods.initialize(mainnetSlot).rpc();
    console.log("tx signature:", tx);
  });
});
