import * as anchor from "@project-serum/anchor";
import {AnchorError, Program} from "@project-serum/anchor";
import {Optional} from "../target/types/optional";
import {expect, assert} from "chai";
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
        await program.methods
            .initialize(new anchor.BN(10), optional2.publicKey)
            .accounts({
                payer: payer.publicKey,
                optional1: optional1.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .signers([optional1]).rpc()

        data1 = await program.account.data1.fetchNullable(optional1.publicKey);

        expect(data1.data.toNumber()).to.equal(10);
    });

    it("realloc_with_constraints", async () => {
        try {
            await program.methods
                .realloc()
                .accounts({
                    payer: payer.publicKey,
                    optional1: optional1.publicKey,
                    optional2: optional2.publicKey,
                    systemProgram: SystemProgram.programId
                })
                .rpc();

            assert.ok(false);
        } catch (e) {
            assert.isTrue(e instanceof AnchorError);
            const err: AnchorError = e;
            const errMsg =
                "A has one constraint was violated";
            assert.strictEqual(err.error.errorMessage, errMsg);
            assert.strictEqual(err.error.errorCode.number,2001);
        }

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
            .signers([optional1, optional2]).rpc()

        let data1 = await program.account.data1.fetchNullable(optional1.publicKey);
        let data2 = await program.account.data2.fetchNullable(optional2.publicKey);
        let data1_info = await program.account.data1.getAccountInfo(optional1.publicKey);


        expect(data1.data.toNumber()).to.equal(10);
        expect(data2.optional1.toString()).to.equal(optional1.publicKey.toString());
        expect(data1_info.data.length).to.equal(16);


        await program.methods
            .realloc()
            .accounts({
                payer: payer.publicKey,
                optional1: optional1.publicKey,
                optional2: optional2.publicKey,
                systemProgram: SystemProgram.programId
            })
            .rpc();

        data1_info = await program.account.data1.getAccountInfo(optional1.publicKey);
        expect(data1_info.data.length).to.equal(20);
    });
});
