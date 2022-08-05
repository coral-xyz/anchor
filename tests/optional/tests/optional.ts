import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {Optional, IDL} from "../target/types/optional";
import { expect } from 'chai';
import {SystemProgram} from "@solana/web3.js";

describe("Optional", () => {
    // configure the client to use the local cluster
    anchor.setProvider(anchor.AnchorProvider.env());

    // let signer = anchor.web3.Keypair.generate();
    const optional1 = anchor.web3.Keypair.generate();
    const optional2 = anchor.web3.Keypair.generate();

    // optional program
    const program = anchor.workspace.Optional as Program<Optional>;
    // payer of the transactions
    const payer = (program.provider as anchor.AnchorProvider).wallet;

    it("initialize", async () => {
        console.log();
        let txn = await program.methods
            .initialize(
                new anchor.BN(10), optional1.publicKey
            )
            .accounts({
                payer: payer.publicKey,
                // optional1: optional1.publicKey,
                systemProgram: SystemProgram.programId,
                optional2: optional2.publicKey,
            })
            // .signers([optional1, optional2])
            .signers([optional2])
        console.log(JSON.stringify((await txn.transaction()).instructions.map(ix => ix.keys), null, " "));
        console.log(JSON.stringify(txn._txFn))
        await txn.rpc();

        let txn2 = await program.methods
            .initialize(
                new anchor.BN(10), optional1.publicKey
            )
            .accounts({
                payer: payer.publicKey,
                optional1: optional1.publicKey,
                systemProgram: SystemProgram.programId,
                // optional2: optional2.publicKey,
            })
            .signers([optional1])
        console.log(JSON.stringify((await txn2.transaction()).instructions.map(ix => ix.keys), null, " "));

        // .signers([optional1, optional2])
            // .signers([optional2])
        // let data1 = await program.account.data1.fetch(optional1.publicKey);
        // console.log(data1)
        let data2 = await program.account.data2.fetch(optional2.publicKey);
        console.log(data2)
        //
        // // expect(data1.data.toNumber()).to.equal(10);
        expect(data2.optional1.toString()).to.equal(optional1.publicKey.toString());
    });

});
