# Transferring Lamports

This collection of recipes will help you understand the various ways to transfer Lamports, the native currency of Solana.
You can learn more about lamports [in the Solana documentation](https://docs.solana.com/terminology#lamport).

## Transferring from an account

```rust
// Create Transaction
let ix = anchor_lang::solana_program::system_instruction::transfer(
    &ctx.accounts.sender.key(), // FROM: the Transaction Sender PubKey
    &ctx.accounts.some_pda.key(), // TO: the PDA PubKey
    amount, // AMOUNT: a u64
);

// Invoke
anchor_lang::solana_program::program::invoke(
    &ix,
    &[
        ctx.accounts.sender.to_account_info(),
        ctx.accounts.some_pda.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ],
)?;
```

To understand what's going on here, it helps to understand that only the Solana Program that owns the account can mutate (or transfer) the lamport balance of an account. By default, accounts are owned by the System Program. In order to transfer any balance, we have to ask the System Program to do so.

::: details
Any account is allowed to receive lamports but ONLY the Program that owns an Account can decrease its balance.
:::

Any System Program lamport transfer also requires that both accounts are mutable and the `sender` must sign the transaction:
```
#[account(mut, signer)]
pub sender: AccountInfo<'info>,
#[accounts(mut)]
pub some_pda: Account<'info, YourPDA>
```

Notice that we do not use `invoke_signed` as the signature on the transaction flows through the Cross Program Invocation to the System Program.


## Transferring from a PDA owned by your Program

```rust
**some_pda.try_borrow_mut_lamports()? = some_pda
    .lamports()
    .checked_sub(amount)
    .ok_or(ProgramError::InvalidArgument)?; // Change or create a relevant Error
**destination.try_borrow_mut_lamports()? = destination
    .lamports()
    .checked_add(amount)
    .ok_or(ProgramError::InvalidArgument)?;
```

This will only work if this is executed by the Program that owns the PDA.

### Transfer all lamports from PDA to an Account
```rust
**ctx.accounts.receiver.try_borrow_mut_lamports()? += ctx.accounts.vault.to_account_info().lamports();
**ctx.accounts.some_pda.to_account_info().try_borrow_mut_lamports()? = 0;
```