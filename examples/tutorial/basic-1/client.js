const anchor = require("@coral-xyz/anchor");
const { SystemProgram } = anchor.web3;

// Configure the cluster.
anchor.setProvider(anchor.AnchorProvider.env());

async function main() {
    // #region main
    // Read the generated IDL.
    const idl = JSON.parse(
      require("fs").readFileSync("./target/idl/basic_1.json", "utf8")
    );
    const provider = anchor.AnchorProvider.env();

    // Address of the deployed program.
    const programId = new anchor.web3.PublicKey("Dysswo9ycPdcCFKsn2NJGRCB9z7FY1rdiJXBhS6iVQB");

    // The Account to create.
    // 実行の度に別々のAccountを生成している
    const myAccount = anchor.web3.Keypair.generate();

    // Generate the program client from IDL.
    const program = new anchor.Program(idl, programId);

    // initializeを実行するtransactionを発行
    // Execute the RPC.
    await program.methods
        .initialize(new anchor.BN(1234))
        .accounts({
            myAccount: myAccount.publicKey,
            user: provider.wallet.publicKey,
            systemProgram: SystemProgram.programId,
        })
        .signers([myAccount])
        .rpc();

    // 上でinitializeを実行して生成したアカウントのデータを書き換えるtransactionを発行
    await program.account.myAccount.fetch(myAccount.publicKey);  // generateしたaccountがfetch出来るまでまたないとエラーになる（試行錯誤して動いたのがこれなので、ベストプラクティスは不明）
    await program.methods
        .update(new anchor.BN(4321))
        .accounts({
          myAccount: myAccount.publicKey,
        })
        .rpc();
    // #endregion main
  }

console.log("Running client.");
main().then(() => console.log("Success"));