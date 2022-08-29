import * as anchor from "@project-serum/anchor";
import {AnchorError, LangErrorMessage, Program, ProgramError, web3} from "@project-serum/anchor";
import {Optional} from "../target/types/optional";
import {assert, expect} from "chai";

describe("Optional", () => {
    // configure the client to use the local cluster
    anchor.setProvider(anchor.AnchorProvider.env());
    let optional2Keypair = web3.Keypair.generate();
    let requiredKeypair1 = web3.Keypair.generate();
    let requiredKeypair2 = web3.Keypair.generate();
    const program = anchor.workspace.Optional as Program<Optional>;

    // payer of the transactions
    const payer = (program.provider as anchor.AnchorProvider).wallet;

    it("initialize", async () => {
        let createRequired = await program.account.data2.createInstruction(
            requiredKeypair1
        );
        // await program.methods
        //     .initialize(new anchor.BN(10), optional2Keypair.publicKey)
        //     .preInstructions([createRequired])
        //     .accounts({
        //         payer: payer.publicKey,
        //         systemProgram: web3.SystemProgram.programId,
        //         required: requiredKeypair1.publicKey,
        //         optional1: null,
        //         optional2: null,
        //     })
        //     .signers([requiredKeypair1])
        //     .rpc({skipPreflight: true});

        // let required1 = await program.account.data2.fetchNullable(
        //     requiredKeypair1.publicKey
        // );
        // expect(required1.optional1.toString()).to.equal(
        //     web3.PublicKey.default.toString()
        // );

        // optional pda
        let seeds = [Buffer.from("data1"), optional2Keypair.publicKey.toBuffer()];
        let optional1Pubkey = web3.PublicKey.findProgramAddressSync(
            seeds,
            program.programId
        )[0];
        let createOptional2 = await program.account.data2.createInstruction(
            optional2Keypair
        );
        let createRequired2 = await program.account.data2.createInstruction(
            requiredKeypair2
        );
        try {
            await program.methods
                .initialize(new anchor.BN(10), optional2Keypair.publicKey)
                .preInstructions([createOptional2, createRequired2])
                .accountsStrict({
                    payer: payer.publicKey,
                    systemProgram: web3.SystemProgram.programId,
                    required: requiredKeypair2.publicKey,
                    optional1: optional1Pubkey,
                    optional2: null,
                })
                .signers([requiredKeypair2, optional2Keypair])
                .rpc();
            expect(false).to.be.true;
        } catch (e) {
            const errMsg1 = "ProgramFailedToComplete";
            const errMsg2 = "Program failed to complete";
            let error: string = e.toString();
            assert(error.includes(errMsg1) || error.includes(errMsg2), "Program didn't fail to complete!!");
        }

        try {
            await program.methods
                .initialize(new anchor.BN(10), optional2Keypair.publicKey)
                .preInstructions([createOptional2, createRequired2])
                .accounts({
                    payer: null,
                    systemProgram: web3.SystemProgram.programId,
                    required: requiredKeypair2.publicKey,
                    optional1: optional1Pubkey,
                    optional2: optional2Keypair.publicKey,
                })
                .signers([requiredKeypair2, optional2Keypair])
                .rpc({skipPreflight: true});
            expect(false).to.be.true;
        } catch (e) {
            console.log(e);
            assert.isTrue(e instanceof ProgramError);
            const err: ProgramError = e;
            assert.strictEqual(err.msg, LangErrorMessage.get(2020));
            assert.strictEqual(err.code, 2020);
        }

        let initValue = new anchor.BN(10);
        await program.methods
            .initialize(initValue, optional2Keypair.publicKey)
            .preInstructions([createOptional2, createRequired2])
            .accounts({
                payer: payer.publicKey,
                systemProgram: web3.SystemProgram.programId,
                required: requiredKeypair2.publicKey,
                optional1: optional1Pubkey,
                optional2: optional2Keypair.publicKey,
            })
            .signers([requiredKeypair2, optional2Keypair])
            .rpc({skipPreflight: true});
        console.log("here")

        let required2 = await program.account.data2.fetchNullable(
            requiredKeypair2.publicKey
        );
        let optional1 = await program.account.data1.fetchNullable(optional1Pubkey);
        let optional2 = await program.account.data2.fetchNullable(
            optional2Keypair.publicKey
        );

        expect(optional1.data.toNumber()).to.equal(initValue.toNumber());
        expect(optional2.optional1.toString()).to.equal(optional1Pubkey.toString());
        expect(required2.optional1.toString()).to.equal(
            web3.PublicKey.default.toString()
        );
    });

    // it("realloc_with_constraints", async () => {
    //     try {
    //         await program.methods
    //             .realloc()
    //             .accounts({
    //                 payer: payer.publicKey,
    //                 optional1: optional1Pubkey,
    //                 required: optional2Keypair.publicKey,
    //                 systemProgram: null,
    //             })
    //             .rpc({skipPreflight: true});
    //
    //         assert.ok(false);
    //     } catch (e) {
    //         assert.isTrue(e instanceof AnchorError);
    //         const err: AnchorError = e;
    //         const errMsg = "A has one constraint was violated";
    //         assert.strictEqual(err.error.errorMessage, errMsg);
    //         assert.strictEqual(err.error.errorCode.number, 2001);
    //     }
    //
    //     optional1 = web3.Keypair.generate();
    //     optional2 = web3.Keypair.generate();
    //     await program.methods
    //         .initialize(new anchor.BN(10), optional2Keypair.publicKey)
    //         .accounts({
    //             payer: payer.publicKey,
    //             optional1: optional1.publicKey,
    //             optional2: optional2Keypair.publicKey,
    //             systemProgram: SystemProgram.programId,
    //         })
    //         .signers([optional1, optional2])
    //         .rpc({skipPreflight: true});
    //
    //     let data1 = await program.account.data1.fetchNullable(optional1.publicKey);
    //     let data2 = await program.account.data2.fetchNullable(optional2Keypair.publicKey);
    //     let data1_info = await program.account.data1.getAccountInfo(
    //         optional1.publicKey
    //     );
    //
    //     expect(data1.data.toNumber()).to.equal(10);
    //     expect(data2.optional1.toString()).to.equal(optional1.publicKey.toString());
    //     expect(data1_info.data.length).to.equal(16);
    //
    //     await program.methods
    //         .realloc()
    //         .accounts({
    //             payer: payer.publicKey,
    //             optional1: optional1.publicKey,
    //             required: optional2Keypair.publicKey,
    //             systemProgram: SystemProgram.programId,
    //         })
    //         .rpc({skipPreflight: true});
    //
    //     data1_info = await program.account.data1.getAccountInfo(
    //         optional1.publicKey
    //     );
    //     expect(data1_info.data.length).to.equal(20);
    // });
});
