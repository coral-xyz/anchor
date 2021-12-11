
## Getting TypeScript Types for the Anchor Program. 

lets say we have a IDL definition for out program like this:

```typescript
export type AnchorVoting = {
  "version": "0.0.0",
  "name": "anchor_voting",
  "instructions": [],
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
  "errors": []
}
```

if you would like to use the account type definition in your code you can use the following code:

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

