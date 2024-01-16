const anchor = require("@coral-xyz/anchor");
const { TransactionInstruction, Connection, SystemProgram, sendAndConfirmTransaction, Transaction } = anchor.web3;

// Configure the cluster.
anchor.setProvider(anchor.AnchorProvider.env());

async function main() {
    // #region main
    // Read the generated IDL.
    const idl = JSON.parse(
      require("fs").readFileSync("./target/idl/basic_1.json", "utf8")
    );
    const provider = anchor.AnchorProvider.env();
    const payer = provider.wallet.payer
    const rpcUrl = provider.connection._rpcEndpoint  // https://api.devnet.solana.com
    const connection = new Connection(rpcUrl, 'confirmed'); // 接続を確立
    // Address of the deployed program.
    const programId = new anchor.web3.PublicKey("Dysswo9ycPdcCFKsn2NJGRCB9z7FY1rdiJXBhS6iVQB");

    // The Account to create.
    // 実行の度に別々のAccountを生成している
    const myAccount = anchor.web3.Keypair.generate();

    const initialValue = 1234
    const byteArray = new Uint8Array(8); // 8バイトの配列を作成
    // bnValue をバイト列に書き込む
    byteArray[0] = (initialValue >> 0) & 0xff;
    byteArray[1] = (initialValue >> 8) & 0xff;
    byteArray[2] = (initialValue >> 16) & 0xff;
    byteArray[3] = (initialValue >> 24) & 0xff;
    byteArray[4] = (initialValue >> 32) & 0xff;
    byteArray[5] = (initialValue >> 40) & 0xff;
    byteArray[6] = (initialValue >> 48) & 0xff;
    byteArray[7] = (initialValue >> 56) & 0xff;

    const instruction1 = new TransactionInstruction(
      {
        keys: [
          {pubkey: myAccount.publicKey, isSigner: true, isWritable: true },  // データアカウント
          {pubkey: payer.publicKey, isSigner: true, isWritable: true },  // マイウォレット
          {pubkey: SystemProgram.programId, isSigner: false, isWritable: false },  // データアカウント
        ],
        programId,
        data: Buffer.from(byteArray),
      }
    )

    await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction1),
      [myAccount, payer],
      "confirmed"
    );

    // transactionを作成
    // const transaction = new Transaction();
    // // initializeのinstructionを追加
    // const instruction1 = await program.methods
    //                               .initialize(new anchor.BN(1234))
    //                               .accounts({
    //                                 myAccount: myAccount.publicKey,
    //                                 user: provider.wallet.publicKey,
    //                                 systemProgram: SystemProgram.programId,
    //                               })
    //                               .signers([myAccount])
    //                               .instruction();
    // transaction.add(instruction1);

    // // updateのinstructionを追加
    // const instruction2 = await program.methods.
    //                             update(new anchor.BN(4321))
    //                             .accounts({
    //                               myAccount: myAccount.publicKey,
    //                             })
    //                             .instruction();
    // transaction.add(instruction2);


    // 接続を確立
    // transactionを署名して送信
    // await connection.sendTransaction(transaction, [myAccount, provider.wallet]);
    // #endregion main
  }

console.log("Running client.");
main().then(() => console.log("Success"));