---
title: Space Reference
description: Anchor - Space Reference
---

This reference tells you how much space you should allocate for an account.

---

This only applies to accounts that don't use `zero-copy`. `zero-copy` uses `repr(C)` with a pointer cast,
so there the `C` layout applies.

In addition to the space for the account data, you have to add `8` to the `space` constraint for Anchor's internal discriminator (see the example).

## Type chart

| Types      | Space in bytes                | Details/Example                                                                                 |
| ---------- | ----------------------------- | ----------------------------------------------------------------------------------------------- |
| bool       | 1                             | would only require 1 bit but still uses 1 byte                                                  |
| u8/i8      | 1                             |
| u16/i16    | 2                             |
| u32/i32    | 4                             |
| u64/i64    | 8                             |
| u128/i128  | 16                            |
| [T;amount] | space(T) \* amount            | e.g. space([u16;32]) = 2 \* 32 = 64                                                             |
| Pubkey     | 32                            |
| Vec\<T>    | 4 + (space(T) \* amount)      | Account size is fixed so account should be initialized with sufficient space from the beginning |
| String     | 4 + length of string in bytes | Account size is fixed so account should be initialized with sufficient space from the beginning |
| Option\<T> | 1 + (space(T))                |
| Enum       | 1 + Largest Variant Size      | e.g. Enum { A, B { val: u8 }, C { val: u16 } } -> 1 + space(u16) = 3                            |
| f32        | 4                             | serialization will fail for NaN                                                                 |
| f64        | 8                             | serialization will fail for NaN                                                                 |

## Example

```rust
#[account]
pub struct MyData {
    pub val: u16,
    pub state: GameState,
    pub players: Vec<Pubkey> // we want to support up to 10 players
}

impl MyData {
    pub const MAX_SIZE: usize = 2 + (1 + 32) + (4 + 10 * 32);
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}

#[derive(Accounts)]
pub struct InitializeMyData<'info> {
    // Note that we have to add 8 to the space for the internal anchor
    #[account(init, payer = signer, space = 8 + MyData::MAX_SIZE)]
    pub acc: Account<'info, MyData>,
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}
```

## The InitSpace macro

Sometimes it can be difficult to calculate the initial space of an account. This macro will add an `INIT_SPACE` constant to the structure. It is not necessary for the structure to contain the `#[account]` macro to generate the constant. Here's an example:

```rust
#[account]
#[derive(InitSpace)]
pub struct ExampleAccount {
    pub data: u64,
    #[max_len(50)]
    pub string_one: String,
    #[max_len(10, 5)]
    pub nested: Vec<Vec<u8>>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(init, payer = payer, space = 8 + ExampleAccount::INIT_SPACE)]
    pub data: Account<'info, ExampleAccount>,
}
```

A few important things to know:

- Don't forget the discriminator when defining `space`
- The `max_len` length represents the length of the structure, not the total length. (ie: the `max_len` of a Vec<u32> will be `max_len` \* 4)
