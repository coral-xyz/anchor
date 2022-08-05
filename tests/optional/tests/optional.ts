import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {Optional} from "../target/types/optional";
import {expect} from "chai";
import {SystemProgram} from "@solana/web3.js";

describe("Optional", () => {
    // configure the client to use the local cluster
    anchor.setProvider(anchor.AnchorProvider.env());
    let optional1 = anchor.web3.Keypair.generate();
    let optional2 = anchor.web3.Keypair.generate();

    // optional program
    const program = anchor.workspace.Optional as Program<Optional>;
    // payer of the transactions
    const payer = (program.provider as anchor.AnchorProvider).wallet;

    it("initialize", async () => {
        await program.methods
            .initialize(new anchor.BN(10), optional2.publicKey)
            .accounts({
                payer: payer.publicKey,
                systemProgram: SystemProgram.programId,
                optional2: optional2.publicKey,
            })
            .signers([optional2]).rpc();

        let data1 = await program.account.data1.fetchNullable(optional1.publicKey);
        let data2 = await program.account.data2.fetchNullable(optional2.publicKey);

        expect(data1).to.equal(null);
        expect(data2.optional1.toString()).to.equal(optional2.publicKey.toString());


        optional1 = anchor.web3.Keypair.generate();
        optional2 = anchor.web3.Keypair.generate();
        await program.methods
            .initialize(new anchor.BN(10), optional2.publicKey)
            .accounts({
                payer: payer.publicKey,
                optional1: optional1.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .signers([optional1]).rpc()

        data1 = await program.account.data1.fetchNullable(optional1.publicKey);
        data2 = await program.account.data2.fetchNullable(optional2.publicKey);

        expect(data2).to.equal(null);
        expect(data1.data.toNumber()).to.equal(10);


        optional1 = anchor.web3.Keypair.generate();
        optional2 = anchor.web3.Keypair.generate();
        await program.methods
            .initialize(new anchor.BN(10), optional2.publicKey)
            .accounts({
                payer: payer.publicKey,
                optional1: optional1.publicKey,
                optional2: optional2.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .signers([optional1]).rpc()

        data1 = await program.account.data1.fetchNullable(optional1.publicKey);
        data2 = await program.account.data2.fetchNullable(optional2.publicKey);

        expect(data1.data.toNumber()).to.equal(10);
        expect(data2.optional1.toString()).to.equal(optional1.publicKey.toString());

    });
});
