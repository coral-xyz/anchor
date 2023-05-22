use anchor_lang::prelude::*;

declare_id!("DuT6R8tQGYa8ACYXyudFJtxDppSALLcmK39b7918jeSC");

#[program]
pub mod basic_5 {
    use super::*;

    pub fn create(ctx: Context<Create>) -> Result<()> {
        let action_state = &mut ctx.accounts.action_state;
        // * - means dereferencing
        action_state.user = *ctx.accounts.user.key;
        // Lets initialize the state
        action_state.action = 0;

        Ok(())
    }
    
    pub fn walk(ctx: Context<Walk>) -> Result<()> {
        let action_state = &mut ctx.accounts.action_state;
        // Lets change the robot action state to "walk"
        action_state.action = 1;

        Ok(())
    }
    
    pub fn run(ctx: Context<Run>) -> Result<()> {
        let action_state = &mut ctx.accounts.action_state;
        // Lets change the robot action state to "run"
        action_state.action = 2;

        Ok(())
    }
    
    pub fn jump(ctx: Context<Jump>) -> Result<()> {
        let action_state = &mut ctx.accounts.action_state;
        // Lets change the robot action state to "jump"
        action_state.action = 3;
        
        Ok(())
    }

    pub fn reset(ctx: Context<Reset>) -> Result<()> {
        let action_state = &mut ctx.accounts.action_state;
        // Lets reset the robot action states
        action_state.action = 0;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Create<'info> {
    // init means to create action_state account
    // bump to use unique address for action_state account
    #[account(init, payer=user, space=ActionState::LEN, seeds=[b"action-state".as_ref(), user.key().as_ref()], bump)]
    pub action_state: Account<'info, ActionState>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Walk<'info> {
    // Only the user on account action_state, should be able to change state
    #[account(mut, has_one = user)]
    pub action_state: Account<'info, ActionState>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Run<'info> {
    // Only the user on account action_state, should be able to change state
    #[account(mut, has_one = user)]
    pub action_state: Account<'info, ActionState>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Jump<'info> {
    // Only the user on account action_state, should be able to change state
    #[account(mut, has_one = user)]
    pub action_state: Account<'info, ActionState>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Reset<'info> {
    // Only the user on account action_state, should be able to change state
    #[account(mut, has_one = user)]
    pub action_state: Account<'info, ActionState>,
    // mut makes it changeble (mutable)
    #[account(mut)]
    pub user: Signer<'info>,
}

#[account]
pub struct ActionState {
    pub user: Pubkey,
    pub action: u8,
}

const DISCRIMINATOR_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const U8_LENGTH: usize = 1;

impl ActionState {
    const LEN: usize = DISCRIMINATOR_LENGTH +
                       PUBLIC_KEY_LENGTH +
                       U8_LENGTH;
}