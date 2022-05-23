// This file is autogenerated with https://github.com/acheroncrypto/native-to-anchor

use anchor_lang::prelude::*;

declare_id!("9NxAd91hhJ3ZBTHytYP894y4ESRKG7n8VbLgdyYGJFLB");

#[program]
pub mod system_program {
    use super::*;

    pub fn create_account(
        ctx: Context<CreateAccount>,
        lamports: u64,
        space: u64,
        owner: Pubkey,
    ) -> Result<()> {
        Ok(())
    }

    pub fn assign(ctx: Context<Assign>, owner: Pubkey) -> Result<()> {
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, lamports: u64) -> Result<()> {
        Ok(())
    }

    pub fn create_account_with_seed(
        ctx: Context<CreateAccountWithSeed>,
        base: Pubkey,
        seed: String,
        lamports: u64,
        space: u64,
        owner: Pubkey,
    ) -> Result<()> {
        Ok(())
    }

    pub fn advance_nonce_account(ctx: Context<AdvanceNonceAccount>) -> Result<()> {
        Ok(())
    }

    pub fn withdraw_nonce_account(ctx: Context<WithdrawNonceAccount>, arg: u64) -> Result<()> {
        Ok(())
    }

    pub fn authorize_nonce_account(ctx: Context<AuthorizeNonceAccount>, arg: Pubkey) -> Result<()> {
        Ok(())
    }

    pub fn allocate(ctx: Context<Allocate>, space: u64) -> Result<()> {
        Ok(())
    }

    pub fn allocate_with_seed(
        ctx: Context<AllocateWithSeed>,
        base: Pubkey,
        seed: String,
        space: u64,
        owner: Pubkey,
    ) -> Result<()> {
        Ok(())
    }

    pub fn assign_with_seed(
        ctx: Context<AssignWithSeed>,
        base: Pubkey,
        seed: String,
        owner: Pubkey,
    ) -> Result<()> {
        Ok(())
    }

    pub fn transfer_with_seed(
        ctx: Context<TransferWithSeed>,
        lamports: u64,
        from_seed: String,
        from_owner: Pubkey,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateAccount<'info> {
    #[account(mut)]
    from: Signer<'info>,
    #[account(mut)]
    to: Signer<'info>,
}

#[derive(Accounts)]
pub struct Assign<'info> {
    #[account(mut)]
    pubkey: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    from: Signer<'info>,
    #[account(mut)]
    /// CHECK:
    to: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateAccountWithSeed<'info> {
    #[account(mut)]
    from: Signer<'info>,
    #[account(mut)]
    /// CHECK:
    to: AccountInfo<'info>,
    base: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdvanceNonceAccount<'info> {
    #[account(mut)]
    /// CHECK:
    nonce: AccountInfo<'info>,
    //recent_blockhashes::id): AccountInfo<'info>,
    //authorized: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawNonceAccount<'info> {
    #[account(mut)]
    /// CHECK:
    nonce: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    to: AccountInfo<'info>,
    //recent_blockhashes::id): AccountInfo<'info>,
    //rent: Sysvar<'info, Rent>,
    //authorized: Signer<'info>,
}

#[derive(Accounts)]
pub struct AuthorizeNonceAccount<'info> {
    #[account(mut)]
    /// CHECK:
    nonce: AccountInfo<'info>,
    authorized: Signer<'info>,
}

#[derive(Accounts)]
pub struct Allocate<'info> {
    #[account(mut)]
    pubkey: Signer<'info>,
}

#[derive(Accounts)]
pub struct AllocateWithSeed<'info> {
    #[account(mut)]
    /// CHECK:
    address: AccountInfo<'info>,
    base: Signer<'info>,
}

#[derive(Accounts)]
pub struct AssignWithSeed<'info> {
    #[account(mut)]
    /// CHECK:
    address: AccountInfo<'info>,
    base: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferWithSeed<'info> {
    #[account(mut)]
    /// CHECK:
    from: AccountInfo<'info>,
    from_base: Signer<'info>,
    #[account(mut)]
    /// CHECK:
    to: AccountInfo<'info>,
}
