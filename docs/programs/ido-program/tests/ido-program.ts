import * as anchor from "@project-serum/anchor";
import { Transaction, SystemProgram, PublicKey } from "@solana/web3.js";
import {
    createAccount,
    createMint,
    getAccount,
    getOrCreateAssociatedTokenAccount,
    mintTo,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { IdoProgram } from "../target/types/ido_program";

describe("ido-program", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);

    const program = anchor.workspace.IdoProgram as Program<IdoProgram>;

    let nativeTokenAmount = new anchor.BN(1000000);

    let fiatMint: PublicKey;
    let redeemableMint: PublicKey;
    let nativeMint: PublicKey;

    let projectFiat: PublicKey;
    let projectNative: PublicKey;

    let investorFiat: PublicKey;
    let investorNative: PublicKey;
    let investorRedeemable: PublicKey;

    let poolNative: PublicKey;
    let poolFiat: PublicKey;

    let poolSigner: PublicKey;

    let nowBn: anchor.BN;
    let startIdoTs: anchor.BN;
    let endIdoTs: anchor.BN;
    let withDrawFiatTs: anchor.BN;

    const payer = anchor.web3.Keypair.generate();
    const mintAuthority = anchor.web3.Keypair.generate();

    const project = anchor.web3.Keypair.generate();
    const investor = anchor.web3.Keypair.generate();

    let pool = anchor.web3.Keypair.generate();

    it("Can initialize the program state", async () => {
        const transferSig = await provider.connection.requestAirdrop(
            payer.publicKey,
            10000000000
        );

        const latestBlockHash = await provider.connection.getLatestBlockhash();

        await provider.connection.confirmTransaction({
            blockhash: latestBlockHash.blockhash,
            lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
            signature: transferSig,
        });

        const tx = new Transaction();

        tx.add(
            SystemProgram.transfer({
                fromPubkey: payer.publicKey,
                toPubkey: mintAuthority.publicKey,
                lamports: 2000000000,
            }),
            SystemProgram.transfer({
                fromPubkey: payer.publicKey,
                toPubkey: project.publicKey,
                lamports: 2000000000,
            }),
            SystemProgram.transfer({
                fromPubkey: payer.publicKey,
                toPubkey: investor.publicKey,
                lamports: 2000000000,
            })
        );

        await provider.sendAndConfirm(tx, [payer]);

        fiatMint = await createMint(
            provider.connection,
            payer,
            mintAuthority.publicKey,
            undefined,
            0,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        nativeMint = await createMint(
            provider.connection,
            payer,
            mintAuthority.publicKey,
            undefined,
            0,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        projectFiat = await createAccount(
            provider.connection,
            payer,
            fiatMint,
            project.publicKey,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        projectNative = await createAccount(
            provider.connection,
            payer,
            nativeMint,
            project.publicKey,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        investorFiat = await createAccount(
            provider.connection,
            payer,
            fiatMint,
            investor.publicKey,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        investorNative = await createAccount(
            provider.connection,
            payer,
            nativeMint,
            investor.publicKey,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        await mintTo(
            provider.connection,
            payer,
            nativeMint,
            projectNative,
            mintAuthority,
            nativeTokenAmount.toNumber()
        );

        await mintTo(
            provider.connection,
            payer,
            fiatMint,
            investorFiat,
            mintAuthority,
            10000
        );

        const projectNativeTokenAccount = await getAccount(
            provider.connection,
            projectNative
        );

        assert.strictEqual(
            projectNativeTokenAccount.amount.toString(),
            nativeTokenAmount.toNumber().toString()
        );
    });

    it("Can initialize the Pool", async () => {
        const [_poolSigner, bump] =
            anchor.web3.PublicKey.findProgramAddressSync(
                [nativeMint.toBuffer()],
                program.programId
            );

        poolSigner = _poolSigner;

        redeemableMint = await createMint(
            provider.connection,
            payer,
            poolSigner,
            undefined,
            0,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        investorRedeemable = await createAccount(
            provider.connection,
            payer,
            redeemableMint,
            investor.publicKey,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID
        );

        let poolNativeAccount = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            payer,
            nativeMint,
            poolSigner,
            true,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID,
            undefined
        );

        poolNative = poolNativeAccount.address;

        let poolFiatAccount = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            payer,
            fiatMint,
            poolSigner,
            true,
            undefined,
            undefined,
            TOKEN_PROGRAM_ID,
            undefined
        );

        poolFiat = poolFiatAccount.address;

        nowBn = new anchor.BN(Date.now() / 1000);
        startIdoTs = nowBn.add(new anchor.BN(10));
        endIdoTs = nowBn.add(new anchor.BN(20));
        withDrawFiatTs = nowBn.add(new anchor.BN(30));

        await program.methods
            .initializePool(
                nativeTokenAmount,
                startIdoTs,
                endIdoTs,
                withDrawFiatTs,
                bump
            )
            .accounts({
                pool: pool.publicKey,
                poolSigner: poolSigner,
                redeemableMint: redeemableMint,
                fiatMint: fiatMint,
                nativeMint: nativeMint,
                poolNative: poolNative,
                poolFiat: poolFiat,
                authority: project.publicKey,
                creatorNative: projectNative,
                tokenProgram: TOKEN_PROGRAM_ID,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([pool, project])
            .rpc();

        const poolNativeTokenAccount = await getAccount(
            provider.connection,
            poolNative
        );

        assert.strictEqual(
            poolNativeTokenAccount.amount.toString(),
            nativeTokenAmount.toNumber().toString()
        );

        const createdPool = await program.account.poolAccount.fetch(
            pool.publicKey
        );

        assert.strictEqual(
            createdPool.poolAuthority.toBase58(),
            project.publicKey.toBase58()
        );
        assert.strictEqual(
            createdPool.redeemableMint.toBase58(),
            redeemableMint.toBase58()
        );
        assert.strictEqual(
            createdPool.poolNative.toBase58(),
            poolNative.toBase58()
        );
        assert.strictEqual(
            createdPool.nativeMint.toBase58(),
            nativeMint.toBase58()
        );
        assert.strictEqual(
            createdPool.poolFiat.toBase58(),
            poolFiat.toBase58()
        );
        assert.strictEqual(
            createdPool.totalNativeTokens.toNumber().toString(),
            nativeTokenAmount.toString()
        );
        assert.strictEqual(
            createdPool.startIdoTs.toNumber().toString(),
            startIdoTs.toString()
        );
        assert.strictEqual(
            createdPool.endIdoTs.toNumber().toString(),
            endIdoTs.toString()
        );
        assert.strictEqual(
            createdPool.withdrawFiatTs.toNumber().toString(),
            withDrawFiatTs.toString()
        );
    });

    let firstDeposit = 5000;

    it("Can exchange investor Fiat for redeemable tokens", async () => {
        if (Date.now() < startIdoTs.toNumber() * 1000) {
            await sleep(startIdoTs.toNumber() * 1000 - Date.now() + 5000);
        }

        await program.methods
            .exchangeFiatForRedeemable(new anchor.BN(firstDeposit))
            .accounts({
                pool: pool.publicKey,
                poolSigner: poolSigner,
                redeemableMint: redeemableMint,
                fiatMint: fiatMint,
                poolFiat: poolFiat,
                authority: investor.publicKey,
                investorFiat: investorFiat,
                investorRedeemable: investorRedeemable,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .signers([investor])
            .rpc();

        const poolFiatTokenAccount = await getAccount(
            provider.connection,
            poolFiat
        );

        assert.strictEqual(
            poolFiatTokenAccount.amount.toString(),
            firstDeposit.toString()
        );

        const investorFiatTokenAccount = await getAccount(
            provider.connection,
            investorFiat
        );

        const investorRedeemableTokenAccount = await getAccount(
            provider.connection,
            investorRedeemable
        );

        assert.strictEqual(
            investorFiatTokenAccount.amount.toString(),
            (10000 - firstDeposit).toString()
        );

        assert.strictEqual(
            investorRedeemableTokenAccount.amount.toString(),
            firstDeposit.toString()
        );
    });

    it("Can exchange investor Redeemable tokens for Native tokens", async () => {
        if (Date.now() < endIdoTs.toNumber() * 1000) {
            await sleep(endIdoTs.toNumber() * 1000 - Date.now() + 5000);
        }

        await program.methods
            .exchangeRedeemableForNative()
            .accounts({
                pool: pool.publicKey,
                poolSigner: poolSigner,
                redeemableMint: redeemableMint,
                poolNative: poolNative,
                authority: investor.publicKey,
                investorNative: investorNative,
                investorRedeemable: investorRedeemable,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .signers([investor])
            .rpc();

        const poolNativeTokenAccount = await getAccount(
            provider.connection,
            poolNative
        );

        assert.strictEqual(poolNativeTokenAccount.amount.toString(), "0");

        const investorNativeTokenAccount = await getAccount(
            provider.connection,
            investorNative
        );

        const investorRedeemableTokenAccount = await getAccount(
            provider.connection,
            investorRedeemable
        );

        assert.strictEqual(
            investorNativeTokenAccount.amount.toString(),
            nativeTokenAmount.toString()
        );

        assert.strictEqual(
            investorRedeemableTokenAccount.amount.toString(),
            "0"
        );
    });

    it("Can withdraw total Fiat from pool account", async () => {
        if (Date.now() < withDrawFiatTs.toNumber() * 1000) {
            await sleep(withDrawFiatTs.toNumber() * 1000 - Date.now() + 5000);
        }

        await program.methods
            .withdrawPoolFiat()
            .accounts({
                pool: pool.publicKey,
                poolSigner: poolSigner,
                fiatMint: fiatMint,
                poolFiat: poolFiat,
                payer: project.publicKey,
                creatorFiat: projectFiat,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            })
            .signers([project])
            .rpc();

        const poolFiatTokenAccount = await getAccount(
            provider.connection,
            poolFiat
        );

        assert.strictEqual(poolFiatTokenAccount.amount.toString(), "0");

        const projectFiatTokenAccount = await getAccount(
            provider.connection,
            projectFiat
        );

        assert.strictEqual(
            projectFiatTokenAccount.amount.toString(),
            firstDeposit.toString()
        );
    });
});

function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}
