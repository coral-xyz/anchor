const anchor = require("@coral-xyz/anchor");
const { bs58, base64 } = require("@coral-xyz/anchor/dist/cjs/utils/bytes");
const { assert } = require("chai");

describe("events", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Events;

  it("Is initialized!", async () => {
    let listener = null;

    let [event, slot] = await new Promise((resolve, _reject) => {
      listener = program.addEventListener("MyEvent", (event, slot) => {
        resolve([event, slot]);
      });
      program.rpc.initialize();
    });
    await program.removeEventListener(listener);

    assert.isAbove(slot, 0);
    assert.strictEqual(event.data.toNumber(), 5);
    assert.strictEqual(event.label, "hello");
  });

  it("Multiple events", async () => {
    // Sleep so we don't get this transaction has already been processed.
    await sleep(2000);

    let listenerOne = null;
    let listenerTwo = null;

    let [eventOne, slotOne] = await new Promise((resolve, _reject) => {
      listenerOne = program.addEventListener("MyEvent", (event, slot) => {
        resolve([event, slot]);
      });
      program.rpc.initialize();
    });

    let [eventTwo, slotTwo] = await new Promise((resolve, _reject) => {
      listenerTwo = program.addEventListener("MyOtherEvent", (event, slot) => {
        resolve([event, slot]);
      });
      program.rpc.testEvent();
    });

    await program.removeEventListener(listenerOne);
    await program.removeEventListener(listenerTwo);

    assert.isAbove(slotOne, 0);
    assert.strictEqual(eventOne.data.toNumber(), 5);
    assert.strictEqual(eventOne.label, "hello");

    assert.isAbove(slotTwo, 0);
    assert.strictEqual(eventTwo.data.toNumber(), 6);
    assert.strictEqual(eventTwo.label, "bye");
  });

  it("Self-CPI events work", async () => {
    await sleep(200);

    let sendTx = await program.transaction.testEventCpi({
      accounts: {
        program: program.programId,
        eventAuthority: anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("__event_authority")],
          program.programId
        )[0],
      },
    });

    let provider = anchor.getProvider();
    let connection = provider.connection;
    let txid = await provider.sendAndConfirm(sendTx, [], {
      commitment: "confirmed",
      skipPreflight: true,
    });

    let tx = await connection.getTransaction(txid, { commitment: "confirmed" });

    let cpiEventData = tx.meta.innerInstructions[0].instructions[0].data;
    let ixData = bs58.decode(cpiEventData);
    let eventData = ixData.slice(8);

    let coder = new anchor.BorshEventCoder(program.idl);
    let event = coder.decode(base64.encode(eventData)).data;

    assert.strictEqual(event.data.toNumber(), 7);
    assert.strictEqual(event.label, "cpi");
  });
});

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
