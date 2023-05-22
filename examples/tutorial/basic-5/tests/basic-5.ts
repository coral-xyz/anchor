import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TransactionInstruction, TransactionMessage, VersionedTransaction } from "@solana/web3.js";
import { Basic5 } from "../target/types/basic_5";

describe("basic-5", () => {
  // Configure the client to use the local cluster.
  //anchor.setProvider(anchor.AnchorProvider.env());
  let provider = anchor.AnchorProvider.local("http://127.0.0.1:8899")

  const program = anchor.workspace.Basic5 as Program<Basic5>;
  const user = anchor.web3.Keypair.generate();

  let [actionState] = anchor.web3.PublicKey.findProgramAddressSync(  
    [anchor.utils.bytes.utf8.encode("action-state"),
    user.publicKey.toBuffer()
    ],
    program.programId);

  before(async () => {

    let res = await provider.connection.requestAirdrop(user.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);

    let latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: res,
    });

  });

  it("basic-5: Robot actions!", async () => {
    // First instruction: set up the Solana accounts to be used
    const instructionOne = await program.methods
        .create()
        .accounts({
          actionState: actionState,
          user: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .instruction();
    // Second instruction: Invoke the Robot to walk
    const instructionTwo = await program.methods
        .walk()
        .accounts({
          actionState: actionState,
          user: user.publicKey,
        })
        .instruction();
    // Third instruction: Invoke the Robot to run    
    const instructionThree = await program.methods
        .run()
        .accounts({
          actionState: actionState,
          user: user.publicKey,
        })
        .instruction();
    // Fourth instruction: Invoke the Robot to jump    
    const instructionFour = await program.methods
        .jump()
        .accounts({
          actionState: actionState,
          user: user.publicKey,
        })
        .instruction();
    // Fifth instruction: Reset actions of the Robot    
    const instructionFive = await program.methods
        .reset()
        .accounts({
          actionState: actionState,
          user: user.publicKey,
        })
        .instruction();

    // Array of instructions
    const instructions: TransactionInstruction[] = [
      instructionOne,
      instructionTwo,
      instructionThree,
      instructionFour,
      instructionFive,
    ];

    createAndSendV0Tx(instructions);
    
  });

  async function createAndSendV0Tx(txInstructions: TransactionInstruction[]) {
      // Step 1 - Fetch Latest Blockhash
      let latestBlockhash = await provider.connection.getLatestBlockhash('confirmed');
      console.log("   ✅ - Fetched latest blockhash. Last Valid Height:", latestBlockhash.lastValidBlockHeight);

      // Step 2 - Generate Transaction Message
      const messageV0 = new TransactionMessage({
        payerKey: user.publicKey,
        recentBlockhash: latestBlockhash.blockhash,
        instructions: txInstructions
      }).compileToV0Message();
      console.log("   ✅ - Compiled Transaction Message");
      const transaction = new VersionedTransaction(messageV0);

      // Step 3 - Sign your transaction with the required `Signers`
      transaction.sign([user]);
      console.log("   ✅ - Transaction Signed");

      // Step 4 - Send our v0 transaction to the cluster
      const txid = await provider.connection.sendTransaction(transaction, { maxRetries: 5 });
      console.log("   ✅ - Transaction sent to network");

      // Step 5 - Confirm Transaction 
      const confirmation = await provider.connection.confirmTransaction({
        signature: txid,
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
      })
      if (confirmation.value.err) { throw new Error("   ❌ - Transaction not confirmed.") }
      //console.log('🎉 Transaction Succesfully Confirmed!', '\n', `https://explorer.solana.com/tx/${txid}?cluster=devnet`);
      console.log('🎉 Transaction Succesfully Confirmed!');
      let result = await program.account.actionState.fetch(actionState);
      console.log("robot action state details: ", result);
  }

});