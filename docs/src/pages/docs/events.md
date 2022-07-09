# Events

Events are an incredibly powerful feature in Anchor. Rather than polling or refreshing to
find out if an account's state has changed, events are like callbacks that save space, compute,
and enable asynchronous programming.

The main downside to events are that they are base64 encoded, and therefore not human readable.
Despite this tradeoff, the UI can easily decode them, and the user benefits from this 
compact mode of logging.


## Event Use Cases

### Return Values
We can use an event to immediately provide a return value, such as details related
to a transaction. Since calls to certain programs only return a tx hash, this may not provide sufficient detail to your UI. An event here can provide a richer experience, by returning values about how a particular transaction was executed, for example.

### Callbacks
 You may want to monitor an event and be notified only once a transaction is confirmed.
Instead of polling / querying the blockchain and looking for an account's values to update, we can build our client to only act when the program has emitted an event, saving on compute resources and fees.

### Compact Storage 
Since events are stored in base64, they can be an economical way to store data
without the trouble of account creation and provisioning for rent-exemption, for example.

## Example

The simplest way to use events is to tack them onto the tail of your program's functions, allowing them to emit
structured data when the function completes without error. 

Start by creating a new anchor project:
```bash
$ anchor init Events
```

Next, we will modify the program in `src/lib.rs` to add an event, and the accompanying data to the `initialize`
function.
```rust
// ...
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        emit!(MyEvent {
            data: 5,
            label: "hello".to_string(),
        });
        Ok(())
    }

// location: src/lib.rs
// ...
#[event]
pub struct MyEvent {
    pub data: u64,
    #[index]
    pub label: String,
}

```

Observe that we are using the event to provide a return value to a successful call to `initialize`.
The data returned is fairly trivial, a `u64` integer and a short `String`. Without the event, we would have no specific information about the outcome of the initialize function, besides that it was successful.

Note that the event is declared with the `#[event]` macro, and defined using a `struct`. To supply the values,
the pattern `emit!(MyEvent {...});` is used. The code above is a basic stamp that can be used for creating and consuming events within your program.

Next, nothing fancy but we'll add a new function called `test_event` with a new event that emits some other values. Complete code for `src/lib.rs` is shown below:

```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod events {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        emit!(MyEvent {
            data: 5,
            label: "hello".to_string(),
        });
        Ok(())
    }

    pub fn test_event(_ctx: Context<TestEvent>) -> Result<()> {
        emit!(MyOtherEvent {
            data: 6,
            label: "bye".to_string(),
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct TestEvent {}

#[event]
pub struct MyEvent {
    pub data: u64,
    #[index]
    pub label: String,
}

#[event]
pub struct MyOtherEvent {
    pub data: u64,
    #[index]
    pub label: String,
}
```

## Consuming Events

To test our program, we'll build a test client that will be able to consume the events. The structure for doing this is essentially:
```javascript
    // ...
    let listener = null;

    let [event, slot] = await new Promise((resolve, _reject) => {
      listener = program.addEventListener("MyEvent", (event, slot) => {
        resolve([event, slot]);
      });
      program.rpc.initialize();
    });
    await program.removeEventListener(listener);
    // ...
```

We set up a `listener` that will catch when the event is emitted by the program. Since it's an asynchronous activity, we must include `await` to receive our return values. We call the first function using `program.rpc.initialize()` and when the event happens, we destructure the output into variables `event` and `slot`. Finally, when it's done, we close the listener. We can access our output values as follows: `event.data.toNumber()` or `event.label`.

With the explanation out of the way, edit the file in `tests/events.ts`. Copy-paste the test code into the file as follows, then run `$ anchor test` from the command line to verify that it's working as intended:

```javascript
import * as anchor from "@project-serum/anchor";
// import { Program } from "@project-serum/anchor";
// import { Events } from "../target/types/events";

const anchor = require("@project-serum/anchor");
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
});

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
```
This is the simplest example for creating and consuming events, but the pattern can be duplicated using dynamic data to provide asynchronous, actionable output for your UI or dApps.