const anchor = require("@coral-xyz/anchor");
const { Transaction, Connection, SystemProgram } = anchor.web3;

// Configure the cluster.
anchor.setProvider(anchor.AnchorProvider.env());

async function main() {
    // #region main
    // Read the generated IDL.
    const idl = JSON.parse(
      require("fs").readFileSync("./target/idl/basic_1.json", "utf8")
    );
    const provider = anchor.AnchorProvider.env();
    const rpcUrl = provider.connection._rpcEndpoint  // https://api.devnet.solana.com
    const connection = new Connection(rpcUrl, 'confirmed'); // 接続を確立

    // Address of the deployed program.
    const programId = new anchor.web3.PublicKey("Dysswo9ycPdcCFKsn2NJGRCB9z7FY1rdiJXBhS6iVQB");

    // The Account to create.
    // 実行の度に別々のAccountを生成している
    const myAccount = anchor.web3.Keypair.generate();

    // Generate the program client from IDL.
    const program = new anchor.Program(idl, programId);

    // transactionを作成
    const transaction = new Transaction();

    // initializeのinstructionを追加
    const instruction1 = await program.methods
                                  .initialize(new anchor.BN(1234))
                                  .accounts({
                                    myAccount: myAccount.publicKey,
                                    user: provider.wallet.publicKey,
                                    systemProgram: SystemProgram.programId,
                                  })
                                  .signers([myAccount])
                                  .instruction();
    transaction.add(instruction1);

    // updateのinstructionを追加
    const instruction2 = await program.methods.
                                update(new anchor.BN(4321))
                                .accounts({
                                  myAccount: myAccount.publicKey,
                                })
                                .instruction();
    transaction.add(instruction2);


    // 接続を確立
    // transactionを署名して送信
    await connection.sendTransaction(transaction, [myAccount, provider.wallet]);
    // #endregion main
  }

console.log("Running client.");
main().then(() => console.log("Success"));