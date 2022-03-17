import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {PythGetPrice} from "../target/types/pyth_get_price";

// Devnet accounts https://pyth.network/markets/?cluster=devnet#Crypto.SOL/USD
const SolProductAccount = new anchor.web3.PublicKey("3Mnn2fX6rQyUsyELYms1sBJyChWofzSNRoqYzvgMVz5E");
const SolPriceAccount = new anchor.web3.PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix")



describe("pyth-get-price", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.Provider.env());

    const program = anchor.workspace.PythGetPrice as Program<PythGetPrice>;

    it("Is initialized!", async () => {
        // Add your test here.
        const tx = await program.rpc.initialize({});
        console.log("Your transaction signature", tx);
    });


    it('should get the SOL/USD price from pyth', async () => {
        const tx = await program.rpc.getPrice({
            accounts: {
                pythProduct: SolProductAccount,
                pythPrice: SolPriceAccount,
            }
        });
        console.log("Your transaction signature",tx)
    });
});
