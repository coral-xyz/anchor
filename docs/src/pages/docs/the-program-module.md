---
title: The Program Module
description: Anchor - The Program Module
---

The program module is where you define your business logic. You do so by writing functions which can be called by clients or other programs. You've already seen one example of such a function, the `set_data` function from the previous section.

---

```rust
#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: u64) -> Result<()> {
        if ctx.accounts.token_account.amount > 0 {
            ctx.accounts.my_account.data = data;
        }
        Ok(())
    }
}
```

## Context

> [Context Reference](https://docs.rs/anchor-lang/latest/anchor_lang/context/index.html)

Each endpoint function takes a `Context` type as its first argument. Through this context argument it can access the accounts (`ctx.accounts`), the program id (`ctx.program_id`) of the executing program, and the remaining accounts (`ctx.remaining_accounts`). 

### Remaining accounts

The context remaining accounts (`ctx.remaining_accounts`) makes it possible to pass dynamic number of accounts in solana. `remaining_accounts` is a vector that contains all accounts that were passed into the instruction but are not declared in the `Accounts` struct. This is useful when you want your function to handle a variable amount of accounts, e.g. when initializing a game with a variable number of players.<br>
Here is an example of transferring lamports from a PDA (program derived account) to dynamic number of user accounts

```rust
#[program]
pub mod transfer {
    use super::*;

     pub fn credit<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, Credit<'info>>, 
        amount: u64
    ) -> Result<()> {
        
        let mut index = 0;
        while index < ctx.remaining_accounts.len() {
            // Distribute the amount equally among all the recepients
            let amount_div = amount/ctx.remaining_accounts.len();
            let from = ctx.accounts.pda.to_account_info();
            let to_account = &ctx.remaining_accounts[index];
            let to = to_account.to_account_info();
            
            let from_lamports = from.lamports();
            let to_lamports = to.lamports();

            // Transfer lamports from PDA to account 
            **to.lamports.borrow_mut() = to_lamports.checked_add(amount_div).unwrap();
            **from.lamports.borrow_mut() = from_lamports.checked_sub(amount_div).unwrap();

            index = index + 1;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Credit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
     #[account(mut, owner = *program_id)]
    pda: Account<'info, Sample>,
    system_program: Program<'info, System>
}

// Following is a sample struct which can be used to create PDA
#[account]
#[derive(Default)]
pub struct Sample {
    bump: u8,
}
```
Calling the method from Typescript client
```typescript
it("Credit !", async () => {
    let amount = 3000000000

    //Consider 3 recepients
    let arr = []
    let recepient = []
    for (let i = 0; i < 3; i++) {
       recepient.push(Keypair.generate());
      let elem = {
        pubkey: recepient[i].publicKey, isWritable: true, isSigner: false
      }
      arr.push(elem)
    }
    
    // PDA formed from seed and payer public key
    const [pda, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("yourPda"), payer.publicKey.toBuffer()], 
      program.programId
    )
    
    const tx = await program.methods.credit(
        new anchor.BN(amount)
    ).accounts({
      pda: pda,
      payer: payer.publicKey,
      systemProgram:  anchor.web3.SystemProgram.programId,
  }).remainingAccounts(arr)
  .signers([payer])
  .rpc()
    
    for (let i = 0; i < 3; i++) {
      let recepientBalance = await provider.connection.getBalance(recepient[i].publicKey)
      expect(recepientBalance).to.equal(amount/3)
   }
    
  });

```
## Instruction Data

If your function requires instruction data, you can add it by adding arguments to the function after the context argument. Anchor will then automatically deserialize the instruction data into the arguments. You can have as many as you like. You can even pass in your own types as long as you use`#[derive(AnchorDeserialize)]` on them or implement `AnchorDeserialize` for them yourself. Here's an example with a custom type used as an instruction data arg:

```rust
#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: Data) -> Result<()> {
        ctx.accounts.my_account.data = data.data;
        ctx.accounts.my_account.age = data.age;
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MyAccount {
    pub data: u64,
    pub age: u8
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Data {
    pub data: u64,
    pub age: u8
}
```

Conveniently, `#[account]` implements `Anchor(De)Serialize` for `MyAccount`, so the example above can be simplified.

```rust
#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: MyAccount) -> Result<()> {
        ctx.accounts.my_account.set_inner(data);
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MyAccount {
    pub data: u64,
    pub age: u8
}
```
