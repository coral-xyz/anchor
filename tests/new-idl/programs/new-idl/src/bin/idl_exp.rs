use anchor_lang::prelude::*;
use std::str::FromStr;

const MY_SEED_U64: u64 = 3;

fn main() {
    let instructions = serde_json::json!({
        "instructions": [
            {
                "name": "initialize",
                "accounts": [
                    {
                        "name": "state",
                        "isMut": true,
                        "isSigner": false,
                        "pda": {
                            "seeds": [
                                {
                                    "kind": "const",
                                    "type": "base58",
                                    "value": bs58::encode(anchor_lang::solana_program::system_program::ID.as_ref()).into_string(),
                                },
                                {
                                    "kind": "const",
                                    "type": "base58",
                                    "value": bs58::encode(Pubkey::from_str("3tMg6nFceRK19FX3WY1Cbtu6DboaabhdVfeYP5BKqkuH").unwrap().as_ref()).into_string(),
                                },
                                {
                                    "kind": "const",
                                    "type": "base58",
                                    "value": bs58::encode(&MY_SEED_U64.to_le_bytes()).into_string(),
                                },
                                {
                                    "kind": "const",
                                    "type": "base58",
                                    "value": bs58::encode(b"some-seed".as_ref()).into_string(),
                                },
                            ],
                        }
                    },
                    {
                        "name": "payer",
                        "isMut": true,
                        "isSigner": true,
                    },
                    {
                        "name": "systemProgram",
                        "isMut": false,
                        "isSigner": false,
                        "pubkey": System::id().to_string()
                    }
                ],
                "args": []
            }
        ]
    });

    println!("{}", serde_json::to_string_pretty(&instructions).unwrap());
}