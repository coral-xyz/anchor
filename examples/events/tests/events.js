const anchor = require('@project-serum/anchor');
const assert = require("assert");

describe("events", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  it("Is initialized!", async () => {
    const program = anchor.workspace.Events;

    let [event, slot] = await new Promise((resolve, _reject) => {
      program.addEventListener("MyEvent", (event, slot) => {
        resolve([event, slot]);
      });
      program.rpc.initialize();
    });
    await program.removeEventListener("MyEvent");

    assert.ok(slot > 0);
    assert.ok(event.data.toNumber() === 5);
    assert.ok(event.label === "hello");
  });
});
