# IDO Pool

This example provides an IDO mechanism that prevents giving "free money" to the quickest bidder. The IDO is selling a fixed amount of 🍉 tokens, guarantees the same price for every user and has three phases:

1. Users can freely deposit and withdraw 💵 tokens into the pool.
2. They can only withdraw 💵 tokens.
3. They can only withdraw 🍉 tokens pro-rata to their share of deposited 💵 tokens to the total amount deposited by all users.

The first two phases should last at least 24 hours each, the last one is unlimited.

## Setup

1. Install dependencies and run the tests to verify it's all working. If
   you are new to anchor you might want to check out the [official guide](https://project-serum.github.io/anchor/getting-started/installation.html).

```
npm install
anchor test
```

2. Create 10 🍉 tokens and 1000 💵 tokens for testing:

```
spl-token create-token --decimals 6
spl-token create-account $MINT_MELON
spl-token mint $MINT_MELON 10 $ACC_MELON

spl-token create-token --decimals 6
spl-token create-account $MINT_USDC
spl-token mint $MINT_USDC 1000 $ACC_USDC
```

3. Deploy the contract and initialize it with 10 🍉 tokens. For testing purposes this pool will only accept deposits for 5 minutes and withdrawals for one more minute afterwards. The default value for both parameters is 24 hours:

```
anchor launch
node cli/index.js init $MINT_USDC $MINT_MELON $ACC_MELON 10 --deposit_duration 300 --cancel_duration 60
```

4. Bid 100 💵 tokens. But first create an account to receive the redeemable pool token, that will allow you to receive 🍉 tokens in phase 3. You can increase or reduce your bid, by calling bid again.

```
spl-token create-account $MINT_REDEEM
node cli/index.js bid $ACC_POOL $ACC_USDC 100 $ACC_REDEEM
```

## Configuration

To use the cli on other clusters than localnet set the env variable CLUSTER_RPC_URL=https://api.devnet.solana.com
