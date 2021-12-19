
## Getting TypeScript Types for the Anchor Program. 

lets say we have a IDL definition for out program like this:

```typescript
export type AnchorVoting = {
  "version": "0.0.0",
  "name": "anchor_voting",
  "instructions": [
    {
      "name": "initializeVoting",
      "accounts": [
        {
          "name": "baseAccount",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "user",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
  ],
  "accounts: [
    {
      "name": "baseAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "totalProposalCount",
            "type": "u64"
          }
        ]
      }
    },
  ],
  "types": [
    {
      "name": "Proposal",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "title",
            "type": "string"
          },
        ]
      }
    }
  ],
  "errors": [
    {
      "code": 300,
      "name": "YouAlreadyVotedForThisProposal",
      "msg": "You have already voted for this proposal"
    },
  ]
}
```

if you would like to use the account type definition in your code you can use the following code:
```rust 
#[account]
pub struct BaseAccount {
    pub total_proposal_count: u64,
}
```

```ts
  import { IdlAccounts } from "@project-serum/anchor";

  interface {
    baseAccount: IdlAccounts<AnchorVoting>["baseAccount"];
  }
```

## Getting struct types from the IDL

Let's say we have a struct definition like this:

```rust
#[derive(Debug,  Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Proposal {
    pub title: String,
}
```

You can access to the struct type definition like this:

```ts
  import { IdlTypes } from "@project-serum/anchor";

  interface {
    proposal: IdlTypes<AnchorVoting>["Proposal"];
  }
```

## Getting error types from the IDL

Let's say we have a error definition like this:

```rust
#[error]
pub enum ErrorCode {
    #[msg("You have already voted for this proposal")]
    YouAlreadyVotedForThisProposal,
}
```

Example of how to access the error type definition:

```ts
  import { IdlErrors} from "@project-serum/anchor";

  interface {
    error: IdlErrors<AnchorVoting>["YouAlreadyVotedForThisProposal"];
  }
```
