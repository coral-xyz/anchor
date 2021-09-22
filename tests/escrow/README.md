# Anchor Example: Escrow Program

## Overview

Since this program is extended from the original [Escrow Program](https://github.com/paul-schaaf/solana-escrow), I assumed you have went through the [original blog post](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/#instruction-rs-part-1-general-code-structure-and-the-beginning-of-the-escrow-program-flow) at least once.

However, there is one major difference between this exmaple and the original Escrow program: Instead of letting initializer create a token account to be reset to a PDA authority, we create a token account `Vault` that has both a PDA key and a PDA authority.

### Initialize

![](https://i.imgur.com/DdociL8.png)

`Initializer` can send a transaction to the escrow program to initialize the Vault. In this transaction, two new accounts: `Vault` and `EscrowAccount`, will be created and tokens (Token A) to be exchanged will be transfered from `Initializer` to `Vault`.

### Cancel

![](https://i.imgur.com/SISvhoy.png)

`Initializer` can also send a transaction to the escrow program to cancel the demand of escrow. The tokens will be transfered back to the `Initialzer` and both `Vault` and `EscrowAccount` will be closed in this case.

### Exchange

![](https://i.imgur.com/h8cNGWS.png)

`Taker` can send a transaction to the escrow to exchange Token B for Token A. First, tokens (Token B) will be transfered from `Taker` to `Initializer`. Afterward, the tokens (Token A) kept in the Vault will be transfered to `Taker`. Finally, both `Vault` and `EscrowAccount` will be closed.

## Build, Deploy and Test

Let's run the test once to see what happens.

First, install dependencies:

```
$ npm install
```

Make sure you have your local solana validator running if you want to deploy the program locally:

```
$ solana-test-validator
```

> If you are on Apple Sillicon M1 chip, you will have to build Solana from the source. See [this document](https://docs.solana.com/cli/install-solana-cli-tools#build-from-source) for more details

Next, we will build and deploy the program via Anchor.

First, let's build the program:

```
$ anchor build
```

Deploy the program:

```
$ anchor deploy
```

Finally, run the test:

```
$ anchor test
```

> Since some features is not supported by the current stable release of Anchor, we will have to run the `anchor-cli` from the source directly.
> Ex:
>
> ```
> $ cargo run --manifest-path ../../cli/Cargo.toml build
> ```

> Maker sure to terminate the `solana-test-validator` before you run the `test` command
