import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {PythGetPrice} from "../target/types/pyth_get_price";
import {assert} from "chai";

// Devnet accounts https://pyth.network/markets/?cluster=devnet#Crypto.SOL/USD
const SolProductAccount = new anchor.web3.PublicKey("3Mnn2fX6rQyUsyELYms1sBJyChWofzSNRoqYzvgMVz5E");
const SolPriceAccount = new anchor.web3.PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix");

const BtcProductAccount = new anchor.web3.PublicKey("3m1y5h2uv7EQL3KaJZehvAJa4yDNvgc5yAdL9KPMKwvk");
const BtcPriceAccount = new anchor.web3.PublicKey("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J");

// TODO: workout how to run pyth tests on localnet

describe("pyth-get-price", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.Provider.env());

    const program = anchor.workspace.PythGetPrice as Program<PythGetPrice>;

    it('should print the SOL/USD price from pyth', async () => {
        const tx = await program.rpc.getPrice({
            accounts: {
                pythProduct: SolProductAccount,
                pythPrice: SolPriceAccount,
            }
        });
        console.log("Your transaction signature", tx)
    });

    it('should price the BTC/USD price from pyth', async () => {
        const tx = await program.rpc.getPrice({
            accounts: {
                pythProduct: BtcProductAccount,
                pythPrice: BtcPriceAccount,
            }
        });
        console.log("Your transaction signature", tx)
    });

    it('should not work for incorrect accounts', async () => {
        try {
            await program.rpc.getPrice({
                accounts: {
                    pythProduct: BtcProductAccount,
                    pythPrice: SolPriceAccount,
                }
            });
            assert.ok(false, "No error was thrown")
        } catch (err) {
            console.log(err.toString())
            assert.equal(err.toString(),"Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: invalid program argument")
        }
    });
});
