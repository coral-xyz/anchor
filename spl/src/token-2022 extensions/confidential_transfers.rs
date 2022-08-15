use anchor_lang::prelude::*;
use anchor_lang::{context::CpiContext,Accounts};
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::solana_program::program::invoke_signed;
use spl_token_2022::extension::confidential_transfer::{
    self,
    ConfidentialTransferMint, 
    DecryptableBalance,
   
};   
use anchor_spl::token::{TokenAccount, Token, Mint,};
pub use spl_token_2022;

pub mod confidential_transfer_wrapper{
    use super::*;
    pub fn initialize_mint<'a,'b,'c,'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, InitializeMint<'info>>,
        confidential_mint: ConfidentialTransferMint
    ) -> Result<()> {
        let ix = confidential_transfer::instruction::initialize_mint(
            &spl_token_2022::ID,
            &ctx.accounts.mint.to_account_info().key,
            &confidential_mint,
        )?;
        invoke_signed(
            &ix,
            &[
                ctx.accounts.mint.to_account_info().clone(),
                ctx.accounts.token_program.to_account_info().clone(),
            ],
            ctx.signer_seeds,
        ).map_err(Into::into)
    }
    pub fn update_mint<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c,'info, UpdateMint<'info>>,
        confidential_mint: ConfidentialTransferMint,
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::update_mint(
            &spl_token_2022::ID, 
            &ctx.accounts.mint.key(), 
            &confidential_mint, 
            &ctx.accounts.authority.to_account_info().key
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.token_program.to_account_info().clone(),
                ctx.accounts.mint.to_account_info().clone(),
            ],
            ctx.signer_seeds
        ).map_err(Into::into)
    }
    

    pub fn approve_account<'a,'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, ApproveAccount<'info>>,
        ) -> Result<()>{
        let ix = confidential_transfer::instruction::approve_account(
            &spl_token_2022::ID, 
            &ctx.accounts.account_to_approve.to_account_info().key, 
            &ctx.accounts.mint.to_account_info().key, 
            &ctx.accounts.authority.to_account_info().key
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.token_program.to_account_info().clone(),
                ctx.accounts.account_to_approve.to_account_info().clone(),
                ctx.accounts.mint.to_account_info().clone(),
            ], 
            ctx.signer_seeds
        )
        .map_err(Into::into)
    }

    // renamed to empty_account for a custom anchor instruction that will wrap
    // confidential_transfer::inner_empty_account instruction as only the "inner" instructions
    // can be CPIed
    pub fn empty_account<'a, 'b, 'c,'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, EmptyAccount<'info>>,
        //multisig_signers: &[&Pubkey],
        proof_instruction_offset: i8,
    ) -> Result<()> {
        let ix = confidential_transfer::instruction::inner_empty_account(
            &spl_token_2022::ID, 
            &ctx.accounts.token_account.to_account_info().key, 
            &ctx.accounts.authority.to_account_info().key, 
            &[], 
            proof_instruction_offset,
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.token_program.to_account_info().clone(),
                ctx.accounts.authority.to_account_info().clone(),
                ctx.accounts.instruction_sysvar_info.to_account_info().clone(),
            ], 
            ctx.signer_seeds
        ).map_err(Into::into)
    }
    pub fn deposit<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b,'c, 'info, Deposit<'info>>,
        amount: u64,
        decimals: u8,
        //multisig_signers: &[&Pubkey], 
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::deposit(
            &spl_token_2022::ID,
            &ctx.accounts.source_token_account.to_account_info().key, 
            &ctx.accounts.mint.to_account_info().key, 
            &ctx.accounts.destination_token_account.to_account_info().key, 
            amount, 
            decimals, 
            &ctx.accounts.authority.key, 
            &[], //multisig_signers
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.token_program.to_account_info().clone(),
                ctx.accounts.source_token_account.to_account_info().clone(),
                ctx.accounts.destination_token_account.to_account_info().clone(),
                ctx.accounts.mint.to_account_info().clone(),
            ], 
            ctx.signer_seeds
        ).map_err(Into::into)
    }
    // renamed to withdraw for a custom anchor instruction that will wrap
    // confidential_transfer::inner_withdraw instruction as only the "inner" instructions
    // can be CPIed
    pub fn withdraw<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, Withdraw<'info>>,
        amount: u64,
        decimals: u8,
        new_decryptable_available_balance: DecryptableBalance,
        proof_instruction_offset: i8,
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::inner_withdraw(
            &spl_token_2022::ID, 
            &ctx.accounts.source_token_account.to_account_info().key, 
            &ctx.accounts.mint.to_account_info().key, 
            &ctx.accounts.destination_token_account.to_account_info().key, 
            amount, 
            decimals, 
            new_decryptable_available_balance, 
            ctx.accounts.authority.to_account_info().key,
            &[], //multisig_signers
            proof_instruction_offset
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.token_program.to_account_info().clone(),
                ctx.accounts.source_token_account.to_account_info().clone(),
                ctx.accounts.destination_token_account.to_account_info().clone(),
                ctx.accounts.mint.to_account_info().clone(),
                ctx.accounts.authority.clone(),
            ],
             ctx.signer_seeds
        ).map_err(Into::into)
    }
    
    // renamed to transfer for a custom anchor instruction that will wrap
    // confidential_transfer::inner_transfer instruction as only the "inner" instructions
    // can be CPIed
    pub fn transfer<'a, 'b, 'c, 'info >(
        ctx: CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>,
        new_decryptable_available_balance: DecryptableBalance,
        proof_instruction_offset: i8,
        //multisig_signers: &[&Pubkey], todo - add support for multisig
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::inner_transfer(
            &spl_token_2022::ID,
            &ctx.accounts.source_token_account.to_account_info().key,
            &ctx.accounts.destination_token_account.to_account_info().key,
            &ctx.accounts.mint.to_account_info().key,
            new_decryptable_available_balance,
            &ctx.accounts.authority.key,
            &[],
            proof_instruction_offset,
        )?;
        invoke_signed(
            &ix,
            &[
                ctx.accounts.source_token_account.to_account_info().clone(),
                ctx.accounts.destination_token_account.to_account_info().clone(),
                ctx.accounts.mint.to_account_info().clone(),
                ctx.accounts.authority.clone(),
            ],
            ctx.signer_seeds
        ).map_err(Into::into)
    }
    

    pub fn enable_balance_credits<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, EnableBalanceCredits<'info>>,
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::enable_balance_credits(
            &spl_token_2022::ID,
            &ctx.accounts.token_account.to_account_info().key,
            &ctx.accounts.authority.key,
            &[],
        )?;
        invoke_signed(
            &ix,
            &[
                ctx.accounts.token_account.to_account_info().clone(),
                ctx.accounts.authority.clone(),
            ],
            ctx.signer_seeds
        ).map_err(Into::into)
    }
    pub fn disable_balance_credits<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, DisableBalanceCredits<'info>>,
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::disable_balance_credits(
            &spl_token_2022::ID,
            &ctx.accounts.token_account.to_account_info().key,
            &ctx.accounts.authority.key,
            &[],
        )?;
        invoke_signed(
            &ix,
            &[
                ctx.accounts.token_account.to_account_info().clone(),
                ctx.accounts.authority.clone(),
            ],
            ctx.signer_seeds
        ).map_err(Into::into)
    }
    pub fn withdraw_withheld_tokens_from_mint<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, WithdrawWithheldTokensFromMint<'info>>,
        proof_instruction_offset: i8,
        // multisig_signers:&[&Pubkey], not added support for multisig yet
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::inner_withdraw_withheld_tokens_from_mint(
            &spl_token_2022::ID,
            &ctx.accounts.mint.to_account_info().key,
            &ctx.accounts.destination_token_account.to_account_info().key,
            &ctx.accounts.authority.key,

            &[],
            proof_instruction_offset,
        )?;
        invoke_signed(
            &ix,
            &[
                ctx.accounts.token_program.to_account_info().clone(),
                ctx.accounts.mint.to_account_info().clone(),
                ctx.accounts.authority.clone(),
            ],
            ctx.signer_seeds
        ).map_err(Into::into)
    }
    
    pub fn withdraw_withheld_tokens_from_accounts<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info,  WithdrawWithheldTokensFromAccounts<'info>>,
        sources: &[&Pubkey],
        proof_instruction_offset: i8,
    ) -> Result<()>{
        let ix = confidential_transfer::instruction::inner_withdraw_withheld_tokens_from_accounts(
            &spl_token_2022::ID, 
            &ctx.accounts.mint.key(), 
            &ctx.accounts.destination_token_account.to_account_info().key, 
            &ctx.accounts.authority.key, 
            &[], 
            sources, 
            proof_instruction_offset
        )?;
        invoke_signed(
            &ix,
            &[
                ctx.accounts.mint.to_account_info().clone(),
                ctx.accounts.destination_token_account.to_account_info().clone(),
                ctx.accounts.instruction_sysvar_info.clone(),
                ctx.accounts.authority.clone()
            ],
            ctx.signer_seeds

        ).map_err(Into::into)


        
    }
    pub fn harvest_withheld_tokens_to_mint<'a,'b,'c,'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, HarvestWithheldTokensToMint<'info>>,
    )-> Result<()>{
        let ix = confidential_transfer::instruction::harvest_withheld_tokens_to_mint(
            &spl_token_2022::ID, 
            &ctx.accounts.mint.to_account_info().key,
            &[],
        )?;
        invoke_signed(
            &ix, 
            &[
                ctx.accounts.mint.to_account_info(),
            ], 
            ctx.signer_seeds,
        ).map_err(Into::into)
    }
}
#[derive(Accounts)]
pub struct InitializeMint<'info>{
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
} 
#[derive(Accounts)]
pub struct UpdateMint<'info>{
    pub mint: Account<'info, Mint>,
    pub new_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ApproveAccount<'info>{
    pub account_to_approve: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub authority: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct EmptyAccount<'info>{
    pub token_account: Account<'info,TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub authority: AccountInfo<'info>,
    pub instruction_sysvar_info: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct Deposit<'info>{
    pub source_token_account: Account<'info, TokenAccount>,
    pub destination_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct Withdraw<'info>{
    pub source_token_account: Account<'info, TokenAccount>,
    pub destination_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub instruction_sysvar_info: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct Transfer<'info>{
    pub source_token_account: Account<'info, TokenAccount>,
    pub destination_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub instruction_sysvar_info: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ApplyPendingBalance<'info>{
    pub token_account: Account<'info, TokenAccount>,
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct EnableBalanceCredits<'info>{
    pub token_account: Account<'info, TokenAccount>,
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct DisableBalanceCredits<'info>{
    pub token_account: Account<'info, TokenAccount>,
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct WithdrawWithheldTokensFromAccounts<'info>{
    pub destination_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub authority: AccountInfo<'info>,
    pub instruction_sysvar_info: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct WithdrawWithheldTokensFromMint<'info>{
    pub destination_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub instruction_sysvar_info: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct HarvestWithheldTokensToMint<'info>{
    mint: Account<'info, Mint>,
}
#[derive(Accounts)]
pub struct HarvestWithheldTokensToAccount<'info>{
    pub mint: Account<'info,  Mint>,
    pub destination_token_account: Account<'info, TokenAccount>,
    pub instruction_sysvar_info: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
}
