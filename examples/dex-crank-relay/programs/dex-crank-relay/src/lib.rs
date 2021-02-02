//! A relatively advanced example. If new to Anchor, it's recommended to start
//! with other examples, first.
//!
//! dex-crank-relay is a proxy program that relays a `ConsumeEvents` instruction
//! to the DEX, counts the number of events processed, and pays out a
//! transaction fee as a function of `fee = fee_rate * num_events_consumed`.
//!
//! To be eligible for the reward, one must first own `stake_threshold` staking
//! pool tokens on the configured staking "registrar".

#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::{self, TokenAccount, Transfer};
use enumflags2::BitFlags;
use registry::{Member, Registrar};
use serum_dex::state::AccountFlag;

#[program]
pub mod dex_crank_relay {
    use super::*;

    pub fn create_reward(ctx: Context<CreateReward>, reward_bucket: RewardBucket) -> Result<()> {
        (*ctx.accounts.reward_bucket) = reward_bucket;
        Ok(())
    }

    pub fn set_stake_threshold(ctx: Context<Auth>, threshold: u64) -> Result<()> {
        ctx.accounts.reward_bucket.stake_threshold = threshold;
        Ok(())
    }

    pub fn set_fee_rate(ctx: Context<Auth>, fee_rate: u64) -> Result<()> {
        ctx.accounts.reward_bucket.fee_rate = fee_rate;
        Ok(())
    }

    pub fn set_authority(ctx: Context<Auth>, new_authority: Pubkey) -> Result<()> {
        ctx.accounts.reward_bucket.authority = new_authority;
        Ok(())
    }

    pub fn set_dex(ctx: Context<Auth>, new_dex_program: Pubkey) -> Result<()> {
        ctx.accounts.reward_bucket.dex_program = new_dex_program;
        Ok(())
    }

    pub fn set_registrar(ctx: Context<Auth>, new_registrar: Pubkey) -> Result<()> {
        ctx.accounts.reward_bucket.registrar = new_registrar;
        Ok(())
    }

    pub fn set_registry_program(ctx: Context<Auth>, new_registry_program: Pubkey) -> Result<()> {
        ctx.accounts.reward_bucket.registry_program = new_registry_program;
        Ok(())
    }

    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        let seeds = [
            ctx.accounts.reward_bucket.to_account_info().key.as_ref(),
            &[ctx.accounts.reward_bucket.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx: CpiContext<Transfer> = (&*ctx.accounts).into();
        token::transfer(cpi_ctx.with_signer(signer), ctx.accounts.vault.amount)?;
        Ok(())
    }

    #[access_control(CrankRelay::accounts(&ctx))]
    pub fn crank_relay<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CrankRelay<'info>>,
        dex_data: Vec<u8>,
    ) -> Result<()> {
        if !is_staked(&ctx) {
            return Err(ErrorCode::InsufficientStake.into());
        }

        // Event queue len before.
        let before_event_count = event_q_len(
            &ctx.accounts
                .dex_event_q
                .to_account_info()
                .try_borrow_data()?,
        );

        // Invoke crank relay.
        {
            let dex_instruction = {
                let relay_meta_accs = ctx
                    .remaining_accounts
                    .iter()
                    .map(|acc_info| {
                        if acc_info.is_writable {
                            AccountMeta::new(*acc_info.key, acc_info.is_signer)
                        } else {
                            AccountMeta::new_readonly(*acc_info.key, acc_info.is_signer)
                        }
                    })
                    .collect::<Vec<AccountMeta>>();
                Instruction {
                    program_id: *ctx.accounts.dex_program.key,
                    accounts: relay_meta_accs,
                    data: dex_data,
                }
            };
            let mut relay_accs = vec![ctx.accounts.dex_program.clone()];
            relay_accs.extend_from_slice(ctx.remaining_accounts);

            solana_program::program::invoke(&dex_instruction, &relay_accs)?;
        }

        // Event queue len after.
        let after_event_count = event_q_len(
            &ctx.accounts
                .dex_event_q
                .to_account_info()
                .try_borrow_data()?,
        );

        // Calculate crank fee.
        let fee = {
            assert!(before_event_count >= after_event_count);
            let num_events = before_event_count - after_event_count;
            let fee = num_events * ctx.accounts.reward_bucket.fee_rate;
            if ctx.accounts.vault.amount < fee {
                msg!("vault depleted");
                return Ok(());
            }
            fee
        };

        // Pay out reward.
        let seeds = [
            ctx.accounts.reward_bucket.to_account_info().key.as_ref(),
            &[ctx.accounts.reward_bucket.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx: CpiContext<Transfer> = (&*ctx.accounts).into();
        token::transfer(cpi_ctx.with_signer(signer), fee)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateReward<'info> {
    #[account(init)]
    reward_bucket: ProgramAccount<'info, RewardBucket>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Auth<'info> {
    #[account(mut, has_one = authority)]
    reward_bucket: ProgramAccount<'info, RewardBucket>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(mut, has_one = vault, has_one = authority)]
    reward_bucket: ProgramAccount<'info, RewardBucket>,
    #[account(seeds = [
        reward_bucket.to_account_info().key.as_ref(),
        &[reward_bucket.nonce],
    ])]
    reward_bucket_signer: AccountInfo<'info>,
    #[account(signer)]
    authority: AccountInfo<'info>,
    #[account(mut)]
    vault: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    to: AccountInfo<'info>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CrankRelay<'info> {
    // Reward bucket.
    #[account(
        has_one = vault,
        has_one = registrar,
        has_one = registry_program,
        has_one = dex_program,
    )]
    reward_bucket: ProgramAccount<'info, RewardBucket>,
    #[account(
        seeds = [
            reward_bucket.to_account_info().key.as_ref(),
            &[reward_bucket.nonce],
        ]
    )]
    reward_bucket_signer: AccountInfo<'info>,
    vault: CpiAccount<'info, TokenAccount>,

    // Stake registry. Since they're CPI accounts, make sure to check owners
    // so that we can avoid actually invoking the CPI and instead just read the
    // accounts.
    registry_program: AccountInfo<'info>,
    #[account("registrar.to_account_info().owner == registry_program.key")]
    registrar: CpiAccount<'info, Registrar>,
    #[account(
        belongs_to = registrar,
        "member.to_account_info().owner == registry_program.key"
    )]
    member: CpiAccount<'info, Member>,
    #[account("member_spt.to_account_info().key == &member.balances.spt")]
    member_spt: CpiAccount<'info, TokenAccount>,
    #[account("member_locked_spt.to_account_info().key == &member.balances_locked.spt")]
    member_locked_spt: CpiAccount<'info, TokenAccount>,

    // DEX.
    #[account("dex_event_q.owner == dex_program.key")]
    dex_event_q: AccountInfo<'info>,
    dex_program: AccountInfo<'info>,

    // Pay reward to.
    #[account(mut)]
    to: AccountInfo<'info>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

impl<'info> CrankRelay<'info> {
    pub fn accounts(ctx: &Context<CrankRelay>) -> Result<()> {
        let data = ctx.accounts.dex_event_q.try_borrow_data()?;

        // b"serum" || account_flags;
        let mut raw_flags = [0u8; 8];
        raw_flags.copy_from_slice(&data[5..13]);
        let account_flags = BitFlags::from_bits(u64::from_le_bytes(raw_flags))
            .map_err(|_| ErrorCode::UnparseableAccountFlags)?;
        if account_flags != (AccountFlag::Initialized | AccountFlag::EventQueue) {
            return Err(ErrorCode::InvalidEventQueue.into());
        }

        Ok(())
    }
}

#[account]
pub struct RewardBucket {
    vault: Pubkey,
    nonce: u8,
    registrar: Pubkey,
    registry_program: Pubkey,
    dex_program: Pubkey,
    authority: Pubkey,
    fee_rate: u64,
    stake_threshold: u64,
}

fn is_staked(ctx: &Context<CrankRelay>) -> bool {
    let total_staked = ctx.accounts.member_spt.amount + ctx.accounts.member_locked_spt.amount;
    let stake_threshold = ctx.accounts.reward_bucket.stake_threshold;
    if total_staked < stake_threshold {
        return false;
    }
    true
}

#[error]
pub enum ErrorCode {
    #[msg("Please stake more to be eligible for crank transaction fees.")]
    InsufficientStake,
    #[msg("Event queue account does not have valid account flags.")]
    InvalidEventQueue,
    #[msg("Unable to parse DEX event queue account flags.")]
    UnparseableAccountFlags,
    Unknown,
}

impl<'a, 'b, 'c, 'info> From<&Migrate<'info>> for CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
    fn from(accounts: &Migrate<'info>) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: accounts.vault.to_account_info().clone(),
            to: accounts.to.to_account_info(),
            authority: accounts.reward_bucket_signer.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'a, 'b, 'c, 'info> From<&CrankRelay<'info>>
    for CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>
{
    fn from(accounts: &CrankRelay<'info>) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: accounts.vault.to_account_info().clone(),
            to: accounts.to.to_account_info(),
            authority: accounts.reward_bucket_signer.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

// Returns the length of the Serum DEX event queue account represented by the
// given `data`.
fn event_q_len(data: &[u8]) -> u64 {
    // b"serum" || account_flags || head.
    let count_start = 5 + 8 + 8;
    let count_end = count_start + 4;
    let mut b = [0u8; 4];
    b.copy_from_slice(&data[count_start..count_end]);
    u32::from_le_bytes(b) as u64
}
