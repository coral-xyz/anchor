import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Transaction, TransactionInstruction, TransactionMessage, VersionedTransaction } from "@solana/web3.js";
import { Basic5 } from "../target/types/basic_5";

describe("basic-5", () => {
  // Configure the client to use the local cluster.
  //anchor.setProvider(anchor.AnchorProvider.env());
  let provider = anchor.AnchorProvider.local("http://127.0.0.1:8899")

  const program = anchor.workspace.Basic5 as Program<Basic5>;
  const user = anchor.web3.Keypair.generate();

  let [action_state] = anchor.web3.PublicKey.findProgramAddressSync(
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

    // Array of instructions
    const instructions: TransactionInstruction[] = [
      // First instruction: set up the Solana accounts to be used
      program.instruction.create({
        accounts: {
          actionState: action_state,
          user: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }),
      // Second instruction: Invoke the Robot to walk
      program.instruction.walk({
        accounts: {
          actionState: action_state,
          user: user.publicKey,
        },
      }),
      // Third instruction: Invoke the Robot to run
      program.instruction.run({
        accounts: {
          actionState: action_state,
          user: user.publicKey,
        },
      }),
      // Fourth instruction: Invoke the Robot to jump
      program.instruction.jump({
        accounts: {
          actionState: action_state,
          user: user.publicKey,
        },
      }),
      // Fifth instruction: Reset actions of the Robot
      program.instruction.reset({
        accounts: {
          actionState: action_state,
          user: user.publicKey,
        },
      }),

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
      let result = await program.account.actionState.fetch(action_state);
      console.log("robot action state details: ", result);
  }

});