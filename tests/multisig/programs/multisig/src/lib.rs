//! An example of a multisig to execute arbitrary Solana transactions.
//!
//! This program can be used to allow a multisig to govern anything a regular
//! Pubkey can govern. One can use the multisig as a BPF program upgrade
//! authority, a mint authority, etc.
//!
//! To use, one must first create a `Multisig` account, specifying two important
//! parameters:
//!
//! 1. Owners - the set of addresses that sign transactions for the multisig.
//! 2. Threshold - the number of signers required to execute a transaction.
//!
//! Once the `Multisig` account is created, one can create a `Transaction`
//! account, specifying the parameters for a normal solana transaction.
//!
//! To sign, owners should invoke the `approve` instruction, and finally,
//! the `execute_transaction`, once enough (i.e. `threshold`) of the owners have
//! signed.

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;
use std::convert::Into;
use std::ops::Deref;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod coral_multisig {
    use super::*;

    // Initializes a new multisig account with a set of owners and a threshold.
    pub fn create_multisig(
        ctx: Context<CreateMultisig>,
        owners: Vec<Pubkey>,
        threshold: u64,
        nonce: u8,
    ) -> Result<()> {
        assert_unique_owners(&owners)?;
        require!(
            threshold > 0 && threshold <= owners.len() as u64,
            MultisigError::InvalidThreshold
        );
        require!(!owners.is_empty(), MultisigError::InvalidOwnersLen);

        let multisig = &mut ctx.accounts.multisig;
        multisig.owners = owners;
        multisig.threshold = threshold;
        multisig.nonce = nonce;
        multisig.owner_set_seqno = 0;
        Ok(())
    }

    // Creates a new transaction account, automatically signed by the creator,
    // which must be one of the owners of the multisig.
    pub fn create_transaction(
        ctx: Context<CreateTransaction>,
        pid: Pubkey,
        accs: Vec<TransactionAccount>,
        data: Vec<u8>,
    ) -> Result<()> {
        let owner_index = ctx
            .accounts
            .multisig
            .owners
            .iter()
            .position(|a| a == ctx.accounts.proposer.key)
            .ok_or(MultisigError::InvalidOwner)?;

        let mut signers = Vec::new();
        signers.resize(ctx.accounts.multisig.owners.len(), false);
        signers[owner_index] = true;

        let tx = &mut ctx.accounts.transaction;
        tx.program_id = pid;
        tx.accounts = accs;
        tx.data = data;
        tx.signers = signers;
        tx.multisig = ctx.accounts.multisig.key();
        tx.did_execute = false;
        tx.owner_set_seqno = ctx.accounts.multisig.owner_set_seqno;

        Ok(())
    }

    // Approves a transaction on behalf of an owner of the multisig.
    pub fn approve(ctx: Context<Approve>) -> Result<()> {
        let owner_index = ctx
            .accounts
            .multisig
            .owners
            .iter()
            .position(|a| a == ctx.accounts.owner.key)
            .ok_or(MultisigError::InvalidOwner)?;

        ctx.accounts.transaction.signers[owner_index] = true;

        Ok(())
    }

    // Set owners and threshold at once.
    pub fn set_owners_and_change_threshold<'info>(
        ctx: Context<'_, '_, '_, 'info, Auth<'info>>,
        owners: Vec<Pubkey>,
        threshold: u64,
    ) -> Result<()> {
        set_owners(
            Context::new(
                ctx.program_id,
                ctx.accounts,
                ctx.remaining_accounts,
                ctx.bumps.clone(),
            ),
            owners,
        )?;
        change_threshold(ctx, threshold)
    }

    // Sets the owners field on the multisig. The only way this can be invoked
    // is via a recursive call from execute_transaction -> set_owners.
    pub fn set_owners(ctx: Context<Auth>, owners: Vec<Pubkey>) -> Result<()> {
        assert_unique_owners(&owners)?;
        require!(!owners.is_empty(), MultisigError::InvalidOwnersLen);

        let multisig = &mut ctx.accounts.multisig;

        if (owners.len() as u64) < multisig.threshold {
            multisig.threshold = owners.len() as u64;
        }

        multisig.owners = owners;
        multisig.owner_set_seqno += 1;

        Ok(())
    }

    // Changes the execution threshold of the multisig. The only way this can be
    // invoked is via a recursive call from execute_transaction ->
    // change_threshold.
    pub fn change_threshold(ctx: Context<Auth>, threshold: u64) -> Result<()> {
        require_gt!(
            threshold,
            ctx.accounts.multisig.owners.len() as u64,
            MultisigError::InvalidThreshold
        );

        let multisig = &mut ctx.accounts.multisig;
        multisig.threshold = threshold;
        Ok(())
    }

    // Executes the given transaction if threshold owners have signed it.
    pub fn execute_transaction(ctx: Context<ExecuteTransaction>) -> Result<()> {
        // Has this been executed already?
        if ctx.accounts.transaction.did_execute {
            return Err(MultisigError::AlreadyExecuted.into());
        }

        // Do we have enough signers.
        let sig_count = ctx
            .accounts
            .transaction
            .signers
            .iter()
            .filter(|&did_sign| *did_sign)
            .count() as u64;
        if sig_count < ctx.accounts.multisig.threshold {
            return Err(MultisigError::NotEnoughSigners.into());
        }

        // Execute the transaction signed by the multisig.
        let mut ix: Instruction = ctx.accounts.transaction.deref().into();
        ix.accounts = ix
            .accounts
            .iter()
            .map(|acc| {
                let mut acc = acc.clone();
                if &acc.pubkey == ctx.accounts.multisig_signer.key {
                    acc.is_signer = true;
                }
                acc
            })
            .collect();
        let multisig_key = ctx.accounts.multisig.key();
        let seeds = &[multisig_key.as_ref(), &[ctx.accounts.multisig.nonce]];
        let signer = &[&seeds[..]];
        let accounts = ctx.remaining_accounts;
        solana_program::program::invoke_signed(&ix, accounts, signer)?;

        // Burn the transaction to ensure one time use.
        ctx.accounts.transaction.did_execute = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateMultisig<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Multisig::INIT_SPACE
    )]
    multisig: Account<'info, Multisig>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTransaction<'info> {
    #[account(mut)]
    payer: Signer<'info>,
    multisig: Account<'info, Multisig>,
    #[account(
        init,
        payer = payer,
        space = 8 + Transaction::INIT_SPACE
    )]
    transaction: Account<'info, Transaction>,
    // One of the owners. Checked in the handler.
    proposer: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(constraint = multisig.owner_set_seqno == transaction.owner_set_seqno)]
    multisig: Account<'info, Multisig>,
    #[account(mut, has_one = multisig)]
    transaction: Account<'info, Transaction>,
    // One of the multisig owners. Checked in the handler.
    owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Auth<'info> {
    #[account(mut)]
    multisig: Account<'info, Multisig>,
    #[account(
        seeds = [multisig.key().as_ref()],
        bump = multisig.nonce,
    )]
    multisig_signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
    #[account(constraint = multisig.owner_set_seqno == transaction.owner_set_seqno)]
    multisig: Account<'info, Multisig>,
    /// CHECK: multisig_signer is a PDA program signer. Data is never read or written to
    #[account(
        seeds = [multisig.key().as_ref()],
        bump = multisig.nonce,
    )]
    multisig_signer: UncheckedAccount<'info>,
    #[account(mut, has_one = multisig)]
    transaction: Account<'info, Transaction>,
}

#[account]
pub struct Multisig {
    #[max_len(5)]
    pub owners: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u8,
    pub owner_set_seqno: u32,
}

#[account]
pub struct Transaction {
    // The multisig account this transaction belongs to.
    pub multisig: Pubkey,
    // Target program to execute against.
    pub program_id: Pubkey,
    // Accounts requried for the transaction.
    #[max_len(20)]
    pub accounts: Vec<TransactionAccount>,
    // Instruction data for the transaction.
    #[max_len(256)]
    pub data: Vec<u8>,
    // signers[index] is true iff multisig.owners[index] signed the transaction.
    #[max_len(5)]
    pub signers: Vec<bool>,
    // Boolean ensuring one time execution.
    pub did_execute: bool,
    // Owner set sequence number.
    pub owner_set_seqno: u32,
}

impl From<&Transaction> for Instruction {
    fn from(tx: &Transaction) -> Instruction {
        Instruction {
            program_id: tx.program_id,
            accounts: tx.accounts.iter().map(Into::into).collect(),
            data: tx.data.clone(),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
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

fn assert_unique_owners(owners: &[Pubkey]) -> Result<()> {
    for (i, owner) in owners.iter().enumerate() {
        require!(
            !owners.iter().skip(i + 1).any(|item| item == owner),
            MultisigError::UniqueOwners
        )
    }
    Ok(())
}

#[error_code]
pub enum MultisigError {
    #[msg("The given owner is not part of this multisig.")]
    InvalidOwner,
    #[msg("Owners length must be non zero.")]
    InvalidOwnersLen,
    #[msg("Not enough owners signed this transaction.")]
    NotEnoughSigners,
    #[msg("Cannot delete a transaction that has been signed by an owner.")]
    TransactionAlreadySigned,
    #[msg("Overflow when adding.")]
    Overflow,
    #[msg("Cannot delete a transaction the owner did not create.")]
    UnableToDelete,
    #[msg("The given transaction has already been executed.")]
    AlreadyExecuted,
    #[msg("Threshold must be less than or equal to the number of owners.")]
    InvalidThreshold,
    #[msg("Owners must be unique")]
    UniqueOwners,
}
