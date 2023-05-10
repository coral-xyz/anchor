use anchor_lang::{prelude::*, solana_program::instruction::Instruction};

#[derive(Debug, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct TransactionInstruction {
    // Target program to execute against.
    pub program_id: Pubkey,
    // Accounts requried for the transaction.
    pub accounts: Vec<TransactionAccount>,
    // Instruction data for the transaction.
    pub data: Vec<u8>,
}

impl From<&TransactionInstruction> for Instruction {
    fn from(tx: &TransactionInstruction) -> Instruction {
        Instruction {
            program_id: tx.program_id,
            accounts: tx.accounts.iter().map(AccountMeta::from).collect(),
            data: tx.data.clone(),
        }
    }
}

impl From<&Instruction> for TransactionInstruction {
    fn from(ix: &Instruction) -> TransactionInstruction {
        TransactionInstruction {
            program_id: ix.program_id,
            accounts: ix.accounts.iter().map(TransactionAccount::from).collect(),
            data: ix.clone().data,
        }
    }
}

impl From<Instruction> for TransactionInstruction {
    fn from(ix: Instruction) -> TransactionInstruction {
        TransactionInstruction {
            program_id: ix.program_id,
            accounts: ix.accounts.iter().map(TransactionAccount::from).collect(),
            data: ix.data,
        }
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransactionAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&TransactionAccount> for AccountMeta {
    fn from(account: &TransactionAccount) -> AccountMeta {
        match account.is_writable {
            false => AccountMeta::new_readonly(account.pubkey, account.is_signer),
            true => AccountMeta::new(account.pubkey, account.is_signer),
        }
    }
}

impl From<&AccountMeta> for TransactionAccount {
    fn from(account_meta: &AccountMeta) -> TransactionAccount {
        TransactionAccount {
            pubkey: account_meta.pubkey,
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        }
    }
}
