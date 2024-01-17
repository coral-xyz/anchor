const anchor = require("@coral-xyz/anchor");
const { TransactionInstruction, Connection, SystemProgram, sendAndConfirmTransaction, Transaction } = anchor.web3;

// Configure the cluster.
anchor.setProvider(anchor.AnchorProvider.env());

function bigintToUint8Array(value) {
  // BigIntをバイナリ文字列に変換
  const binaryString = value.toString(2);

  // バイナリ文字列を8ビットごとに分割
  const paddedBinaryString = binaryString.padStart(8 * Math.ceil(binaryString.length / 8), '0');
  const chunks = paddedBinaryString.match(/.{8}/g);

  // 各8ビットチャンクをUint8Arrayに変換
  const uint8Array = new Uint8Array(chunks.length);
  for (let i = 0; i < chunks.length; i++) {
    uint8Array[i] = parseInt(chunks[i], 2);
  }

  return uint8Array;
}

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

    // initializeのinstructionの作成
    const inst1_initializeSighash = [0xaf, 0xaf, 0x6d, 0x1f, 0x0d, 0x98, 0x9b, 0xed]  // https://solana.stackexchange.com/questions/4948/what-is-anchor-8-bytes-discriminator
    const inst1_initialValue = BigInt(1234)  // BigNumber
    const inst1_byteArray = new Uint8Array(16); // 16バイトの配列を作成（前半8バイトはsighash、後半はinitializeの引数）
    // sighashをバイト列に書き込む
    inst1_byteArray[0] = inst1_initializeSighash[0]
    inst1_byteArray[1] = inst1_initializeSighash[1]
    inst1_byteArray[2] = inst1_initializeSighash[2]
    inst1_byteArray[3] = inst1_initializeSighash[3]
    inst1_byteArray[4] = inst1_initializeSighash[4]
    inst1_byteArray[5] = inst1_initializeSighash[5]
    inst1_byteArray[6] = inst1_initializeSighash[6]
    inst1_byteArray[7] = inst1_initializeSighash[7]
    // initialValue をバイト列に書き込む
    inst1_byteArray[8] = bigintToUint8Array((inst1_initialValue >> 0n) & BigInt(0xff));
    inst1_byteArray[9] = bigintToUint8Array((inst1_initialValue >> 8n) & BigInt(0xff));
    inst1_byteArray[10] = bigintToUint8Array((inst1_initialValue >> 16n) & BigInt(0xff));
    inst1_byteArray[11] = bigintToUint8Array((inst1_initialValue >> 24n) & BigInt(0xff));
    inst1_byteArray[12] = bigintToUint8Array((inst1_initialValue >> 32n) & BigInt(0xff));
    inst1_byteArray[13] = bigintToUint8Array((inst1_initialValue >> 40n) & BigInt(0xff));
    inst1_byteArray[14] = bigintToUint8Array((inst1_initialValue >> 48n) & BigInt(0xff));
    inst1_byteArray[15] = bigintToUint8Array((inst1_initialValue >> 56n) & BigInt(0xff));

    const instruction1 = new TransactionInstruction(
      {
        keys: [
          {pubkey: payer.publicKey, isSigner: true, isWritable: true },  // マイウォレット
          {pubkey: myAccount.publicKey, isSigner: true, isWritable: true },  // データアカウント
          {pubkey: SystemProgram.programId, isSigner: false, isWritable: false },  // データアカウント
          {pubkey: programId, isSigner: false, isWritable: false },  // プログラムアカウント
        ],
        programId,
        data: Buffer.from(inst1_byteArray),
      }
    )

    await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction1),
      [payer, myAccount],
      "confirmed"
    );

    // #endregion main
  }

console.log("Running client.");
main().then(() => console.log("Success"));