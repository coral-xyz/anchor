import * as utils from "../src/utils";
import { Keypair } from "@solana/web3.js";

describe("seeds-tests", () => {
  it("seeds test", async () => {
    //given
    const stringSeed = "fafarafa";
    const payer = Keypair.generate();

    //when
    const seedsOld = [
      payer.publicKey.toBuffer(),
      Buffer.from(utils.bytes.utf8.encode(stringSeed)),
    ];
    const seedsPartiallyNew = [
      payer.publicKey.toBuffer(),
      ...utils.seeds.from(stringSeed),
    ];
    const seedsNew = utils.seeds.from(payer.publicKey, stringSeed);

    //then
    expect(seedsOld).toEqual(seedsNew);
    expect(seedsPartiallyNew).toEqual(seedsNew);
  });
});
