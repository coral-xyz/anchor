# Introduction

Anchor is a framework for Solana's [Sealevel](https://medium.com/solana-labs/sealevel-parallel-processing-thousands-of-smart-contracts-d814b378192) runtime providing several convenient developer tools.

- Rust crates and eDSL for writing Solana programs
- [IDL](https://en.wikipedia.org/wiki/Interface_description_language) specification
- TypeScript package for generating clients from IDL
- CLI and workspace management for developing complete applications

If you're familiar with developing in Ethereum's [Solidity](https://docs.soliditylang.org/en/v0.7.4/), [Truffle](https://www.trufflesuite.com/), [web3.js](https://github.com/ethereum/web3.js) or Parity's [Ink!](https://github.com/paritytech/ink), then the experience will be familiar. Although the DSL syntax and semantics are targeted at Solana, the high level flow of writing RPC request handlers, emitting an IDL, and generating clients from IDL is the same.

Here, we'll walkthrough several tutorials demonstrating how to use Anchor. To skip the tutorials and jump straight to a full example, checkout the
[quickstart](./quick-start.md) or go [here](https://github.com/project-serum/anchor/blob/master/examples/tutorial/basic-2/programs/basic-2/src/lib.rs). For an introduction to Solana, see the [docs](https://docs.solana.com/developing/programming-model/overview).

::: tip NOTE
Anchor is in active development, so all APIs are subject to change. If you are one of the early developers to try it out and have feedback, please reach out by [filing an issue](https://github.com/project-serum/anchor/issues/new). This documentation is a work in progress and is expected to change dramatically as features continue to be built out. If you have any problems, consult the [source](https://github.com/project-serum/anchor) or feel free to ask questions on the [Serum Discord](https://discord.com/channels/739225212658122886/752530209848295555).
:::
