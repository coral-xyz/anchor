#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Mint, MintTo, TokenAccount};
use std::convert::Into;

#[program]
mod registry {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        mint: Pubkey,
        authority: Pubkey,
        nonce: u8,
        withdrawal_timelock: i64,
        max_stake: u64,
        stake_rate: u64,
    ) -> Result<(), Error> {
        /*
                let event_q =
                    RewardEventQueue::from(ctx.accounts.reward_event_q.to_account_info().data.clone());
                if event_q.get_init()? {
                    return Err(ErrorCode::RewardQAlreadyInitialized.into());
                }
        */
        let vault_authority = Pubkey::create_program_address(
            &spt_signer_seeds(ctx.accounts.registrar.to_account_info().key, &nonce),
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidNonce)?;
        if ctx.accounts.pool_mint.mint_authority != COption::Some(vault_authority) {
            return Err(ErrorCode::InvalidPoolMintAuthority.into());
        }

        let registrar = &mut ctx.accounts.registrar;

        registrar.authority = authority;
        registrar.nonce = nonce;
        registrar.mint = mint;
        registrar.pool_mint = *ctx.accounts.pool_mint.to_account_info().key;
        registrar.stake_rate = stake_rate;
        registrar.reward_event_q = *ctx.accounts.reward_event_q.to_account_info().key;
        registrar.withdrawal_timelock = withdrawal_timelock;
        registrar.max_stake = max_stake;

        //        event_q.set_init()?;
        //        event_q.set_authority(registrar.to_account_info().key);

        Ok(())
    }

    pub fn update_registrar(
        ctx: Context<UpdateRegistrar>,
        new_authority: Option<Pubkey>,
        withdrawal_timelock: Option<i64>,
        max_stake: Option<u64>,
    ) -> Result<(), Error> {
        let registrar = &mut ctx.accounts.registrar;

        if let Some(new_authority) = new_authority {
            registrar.authority = new_authority;
        }

        if let Some(withdrawal_timelock) = withdrawal_timelock {
            registrar.withdrawal_timelock = withdrawal_timelock;
        }

        if let Some(max_stake) = max_stake {
            registrar.max_stake = max_stake;
        }

        Ok(())
    }

    pub fn create_member(ctx: Context<CreateMember>, nonce: u8) -> Result<(), Error> {
        let seeds = &[
            ctx.accounts.registrar.to_account_info().key.as_ref(),
            ctx.accounts.member.to_account_info().key.as_ref(),
            &[nonce],
        ];
        let signer = &[&seeds[..]];

        // Check the nonce + signer is correct.
        let member_signer = Pubkey::create_program_address(seeds, ctx.program_id)
            .map_err(|_| ErrorCode::InvalidNonce)?;
        if &member_signer != ctx.accounts.member_signer.to_account_info().key {
            return Err(ErrorCode::InvalidMemberSigner.into());
        }

        // Initialize member.
        let member = &mut ctx.accounts.member;
        member.registrar = *ctx.accounts.registrar.to_account_info().key;
        member.beneficiary = *ctx.accounts.beneficiary.key;
        member.balances = ctx.accounts.balances.clone().into();
        member.balances_locked = ctx.accounts.balances_locked.clone().into();
        member.nonce = nonce;

        // Set delegate on staking tokens.
        let (spt_approve, locked_spt_approve) = {
            (
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.clone(),
                    token::Approve {
                        to: ctx.accounts.balances.spt.to_account_info(),
                        delegate: ctx.accounts.beneficiary.to_account_info(),
                        authority: ctx.accounts.member_signer.to_account_info(),
                    },
                    signer,
                ),
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.clone(),
                    token::Approve {
                        to: ctx.accounts.balances_locked.spt.to_account_info(),
                        delegate: ctx.accounts.beneficiary.to_account_info(),
                        authority: ctx.accounts.member_signer.to_account_info(),
                    },
                    signer,
                ),
            )
        };
        token::approve(spt_approve, 0)?;
        token::approve(locked_spt_approve, 0)?;

        Ok(())
    }

    pub fn update_member(
        ctx: Context<UpdateMember>,
        metadata: Option<Pubkey>,
    ) -> Result<(), Error> {
        let member = &mut ctx.accounts.member;
        if let Some(m) = metadata {
            member.metadata = m;
        }
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<(), Error> {
        token::transfer(ctx.accounts.into(), amount).map_err(Into::into)
    }

    #[access_control(no_available_rewards(&ctx))]
    pub fn stake(ctx: Context<Stake>, spt_amount: u64, balance_id: Pubkey) -> Result<(), Error> {
        // Choose balances (locked or unlocked) based on balance_id.
        let balances = {
            if balance_id == ctx.accounts.member.beneficiary {
                &ctx.accounts.balances
            } else {
                &ctx.accounts.balances_locked
            }
        };

        // Transfer tokens into the stake vault.
        {
            // Convert from stake-token units to mint-token units.
            let token_amount = spt_amount
                .checked_mul(ctx.accounts.registrar.stake_rate)
                .unwrap();

            let seeds = &[
                ctx.accounts.registrar.to_account_info().key.as_ref(),
                ctx.accounts.member.to_account_info().key.as_ref(),
                &[ctx.accounts.member.nonce],
            ];
            let member_signer = &[&seeds[..]];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.clone(),
                token::Transfer {
                    from: balances.vault.to_account_info(),
                    to: balances.vault_stake.to_account_info(),
                    authority: ctx.accounts.member_signer.to_account_info(),
                },
                member_signer,
            );
            token::transfer(cpi_ctx, token_amount)?;
        }

        // Mint pool tokens to the staker.
        {
            let seeds = &spt_signer_seeds(
                ctx.accounts.registrar.to_account_info().key,
                &ctx.accounts.registrar.nonce,
            );
            let registrar_signer = &[&seeds[..]];

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.clone(),
                token::MintTo {
                    mint: ctx.accounts.pool_mint.to_account_info(),
                    to: balances.spt.to_account_info(),
                    authority: ctx.accounts.member_signer.to_account_info(),
                },
                registrar_signer,
            );
            token::mint_to(cpi_ctx, spt_amount)?;
        }

        Ok(())
    }

    pub fn start_unstake(
        ctx: Context<StartUnstake>,
        spt_amount: u64,
        balance_id: Pubkey,
    ) -> Result<(), Error> {
        // todo
        Ok(())
    }

    pub fn end_unstake(ctx: Context<EndUnstake>) -> Result<(), Error> {
        // todo
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<(), Error> {
        // todo
        Ok(())
    }

    pub fn drop_reward(ctx: Context<DropReward>) -> Result<(), Error> {
        // todo
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<(), Error> {
        // todo
        Ok(())
    }

    pub fn expire_reward(ctx: Context<ExpireReward>) -> Result<(), Error> {
        // todo
        Ok(())
    }
}

// Asserts the user calling the `Stake` instruction has no rewards available
// in the reward queue.
pub fn no_available_rewards(ctx: &Context<Stake>) -> Result<(), Error> {
    // todo
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init)]
    registrar: ProgramAccount<'info, Registrar>,
    pool_mint: CpiAccount<'info, Mint>,
    //    #[account(owner = program)]
    reward_event_q: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateRegistrar<'info> {
    #[account(mut, has_one = authority)]
    registrar: ProgramAccount<'info, Registrar>,
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateMember<'info> {
    registrar: ProgramAccount<'info, Registrar>,
    #[account(init)]
    member: ProgramAccount<'info, Member>,
    #[account(signer)]
    beneficiary: AccountInfo<'info>,
    // Must be verified against the user given nonce.
    member_signer: AccountInfo<'info>,
    #[account(
        "&balances.spt.owner == member_signer.key",
        "balances.spt.mint == registrar.pool_mint",
        "balances.vault.mint == registrar.mint"
    )]
    balances: BalanceSandboxAccounts<'info>,
    #[account(
        "&balances_locked.spt.owner == member_signer.key",
        "balances_locked.spt.mint == registrar.pool_mint",
        "balances_locked.vault.mint == registrar.mint"
    )]
    balances_locked: BalanceSandboxAccounts<'info>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
}

// Invariants:
//
// * owner of all vaults is the registrar signer
// * token mint is one of the registrar's mints
// * pool token mint is one of the registrar's pool token mints
//
#[derive(Accounts, Clone)]
pub struct BalanceSandboxAccounts<'info> {
    balance_id: AccountInfo<'info>,
    #[account(mut, "spt.delegate == COption::None")]
    spt: CpiAccount<'info, TokenAccount>,
    #[account("vault.owner == spt.owner")]
    vault: CpiAccount<'info, TokenAccount>,
    #[account("vault_stake.owner == spt.owner", "vault_stake.mint == vault.mint")]
    vault_stake: CpiAccount<'info, TokenAccount>,
    #[account("vault_pw.owner == spt.owner", "vault_pw.mint == vault.mint")]
    vault_pw: CpiAccount<'info, TokenAccount>,
}

impl<'info> From<BalanceSandboxAccounts<'info>> for BalanceSandbox {
    fn from(accs: BalanceSandboxAccounts<'info>) -> Self {
        Self {
            balance_id: *accs.balance_id.key,
            spt: *accs.spt.to_account_info().key,
            vault: *accs.vault.to_account_info().key,
            vault_stake: *accs.vault_stake.to_account_info().key,
            vault_pw: *accs.vault_pw.to_account_info().key,
        }
    }
}

#[derive(Accounts)]
pub struct UpdateMember<'info> {
    #[account(mut, has_one = beneficiary)]
    member: ProgramAccount<'info, Member>,
    #[account(signer)]
    beneficiary: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // Lockup whitelist relay interface.
    _vesting: AccountInfo<'info>,
    #[account(mut)]
    depositor: AccountInfo<'info>,
    #[account(signer)]
    depositor_authority: AccountInfo<'info>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    #[account(mut, "&member_vault.owner == member_vault_authority.key")]
    member_vault: CpiAccount<'info, TokenAccount>,
    #[account(
				seeds = [
						registrar.to_account_info().key.as_ref(),
						member.to_account_info().key.as_ref(),
						&[member.nonce],
				]
		)]
    member_vault_authority: AccountInfo<'info>,

    // Program specific.
    registrar: ProgramAccount<'info, Registrar>,
    #[account(signer)]
    beneficiary: AccountInfo<'info>,
    #[account(belongs_to = registrar, belongs_to = beneficiary)]
    member: ProgramAccount<'info, Member>,
}

impl<'a, 'b, 'c, 'info> From<&mut Deposit<'info>>
    for CpiContext<'a, 'b, 'c, 'info, anchor_spl::token::Transfer<'info>>
{
    fn from(
        accounts: &mut Deposit<'info>,
    ) -> CpiContext<'a, 'b, 'c, 'info, anchor_spl::token::Transfer<'info>> {
        let cpi_accounts = anchor_spl::token::Transfer {
            from: accounts.depositor.clone(),
            to: accounts.member_vault.to_account_info(),
            authority: accounts.depositor_authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    // Global accounts for the staking instance.
    #[account(has_one = pool_mint, has_one = reward_event_q)]
    registrar: ProgramAccount<'info, Registrar>,
    #[account(mut)]
    reward_event_q: AccountInfo<'info>,
    #[account(mut)]
    pool_mint: CpiAccount<'info, Mint>,

    // Member specific.
    #[account(mut, has_one = beneficiary, belongs_to = registrar)]
    member: ProgramAccount<'info, Member>,
    #[account(signer)]
    beneficiary: AccountInfo<'info>,
    #[account(
        "&balances.spt.owner == member_signer.key",
        "balances.spt.mint == registrar.pool_mint",
        "balances.vault.mint == registrar.mint"
    )]
    balances: BalanceSandboxAccounts<'info>,
    #[account(
        "&balances_locked.spt.owner == member_signer.key",
        "balances_locked.spt.mint == registrar.pool_mint",
        "balances_locked.vault.mint == registrar.mint"
    )]
    balances_locked: BalanceSandboxAccounts<'info>,

    // Programmatic signers.
    #[account(
				seeds = [
						registrar.to_account_info().key.as_ref(),
						member.to_account_info().key.as_ref(),
						&[member.nonce],
				],
		)]
    member_signer: AccountInfo<'info>,
    #[account(
				seeds = [
						registrar.to_account_info().key.as_ref(),
						&[registrar.nonce],
				]
		)]
    registrar_signer: AccountInfo<'info>,

    // Misc.
    clock: Sysvar<'info, Clock>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct StartUnstake {
    // todo
}

#[derive(Accounts)]
pub struct EndUnstake {
    // todo
}

#[derive(Accounts)]
pub struct Withdraw {
    // todo
}

#[derive(Accounts)]
pub struct DropReward {
    // todo
}

#[derive(Accounts)]
pub struct ClaimReward {
    // todo
}

#[derive(Accounts)]
pub struct ExpireReward {
    // todo
}

#[account]
pub struct Registrar {
    /// Priviledged account.
    pub authority: Pubkey,
    /// Nonce to derive the program-derived address owning the vaults.
    pub nonce: u8,
    /// The maximum stake per member, denominated in the mint.
    pub max_stake: u64,
    /// Number of seconds that must pass for a withdrawal to complete.
    pub withdrawal_timelock: i64,
    /// Global event queue for reward vendoring.
    pub reward_event_q: Pubkey,
    /// Mint of the tokens that can be staked.
    pub mint: Pubkey,
    /// Staking pool token mint.
    pub pool_mint: Pubkey,
    /// The amount of tokens (not decimal) that must be staked to get a single
    /// staking pool token.
    pub stake_rate: u64,
}

#[account]
pub struct Member {
    /// Registrar the member belongs to.
    pub registrar: Pubkey,
    /// The effective owner of the Member account.
    pub beneficiary: Pubkey,
    /// Arbitrary metadata account owned by any program.
    pub metadata: Pubkey,
    /// Sets of balances owned by the Member.
    pub balances: BalanceSandbox,
    /// Locked balances owned by the Member.
    pub balances_locked: BalanceSandbox,
    /// Next position in the rewards event queue to process.
    pub rewards_cursor: u32,
    /// The clock timestamp of the last time this account staked or switched
    /// entities. Used as a proof to reward vendors that the Member account
    /// was staked at a given point in time.
    pub last_stake_ts: i64,
    /// Signer nonce.
    pub nonce: u8,
}

// BalanceSandbox defines isolated funds that can only be deposited/withdrawn
// into the program if the `owner` signs off on the transaction.
//
// Once controlled by the program, the associated `Member` account's beneficiary
// can send funds to/from any of the accounts within the sandbox, e.g., to
// stake.
#[derive(AnchorSerialize, AnchorDeserialize, Default, Debug, Clone)]
pub struct BalanceSandbox {
    pub balance_id: Pubkey,
    // Staking pool token.
    pub spt: Pubkey,
    // Free balance (deposit) vaults.
    pub vault: Pubkey,
    // Stake vaults.
    pub vault_stake: Pubkey,
    // Pending withdrawal vaults.
    pub vault_pw: Pubkey,
}

/*
#[account]
pub struct PendingWithdrawal {
    /// Member this account belongs to.
    pub member: Pubkey,
    /// One time token. True if the withdrawal has been completed.
    pub burned: bool,
    /// The pool being withdrawn from.
    pub pool: Pubkey,
    /// Unix timestamp when this account was initialized.
    pub start_ts: i64,
    /// Timestamp when the pending withdrawal completes.
    pub end_ts: i64,
    /// The number of tokens redeemed from the staking pool.
    pub amount: u64,
    /// The Member account's set of vaults this withdrawal belongs to.
    pub balance_id: Pubkey,
}

#[account]
pub struct LockedRewardVendor {
    pub registrar: Pubkey,
    pub vault: Pubkey,
    pub nonce: u8,
    pub pool: Pubkey,
    pub pool_token_supply: u64,
    pub reward_event_q_cursor: u32,
    pub start_ts: i64,
    pub end_ts: i64,
    pub expiry_ts: i64,
    pub expiry_receiver: Pubkey,
    pub total: u64,
    pub period_count: u64,
    pub expired: bool,
}

#[account]
pub struct UnlockedRewardVendor {
    pub registrar: Pubkey,
    pub vault: Pubkey,
    pub nonce: u8,
    pub pool: Pubkey,
    pub pool_token_supply: u64,
    pub reward_event_q_cursor: u32,
    pub start_ts: i64,
    pub expiry_ts: i64,
    pub expiry_receiver: Pubkey,
    pub total: u64,
    pub expired: bool,
}
*/

/*
pub fn stake_will_max(&self, spt_amount: u64, registrar: &Registrar) -> bool {
    let spt_value = spt_amount
        .checked_mul(registrar.stake_rate)
        .unwrap()
        .checked_add(self.spt.checked_mul(registrar.stake_rate).unwrap())
        .unwrap();
    spt_value > registrar.max_stake
}
 */

fn spt_signer_seeds<'a>(registrar: &'a Pubkey, nonce: &'a u8) -> [&'a [u8]; 2] {
    [registrar.as_ref(), bytemuck::bytes_of(nonce)]
}

// Largest reward variant size.
//
// Don't forget to change the typescript when modifying this.
const MAX_RING_ITEM_SIZE: u32 = 145;
/*

// Generate the Ring trait.
serum_common::ring!(MAX_RING_ITEM_SIZE);

pub struct RewardEventQueue<'a> {
    pub storage: Rc<RefCell<&'a mut [u8]>>,
}

impl<'a> RewardEventQueue<'a> {
    // Don't forget to change the typescript when modifying this.
    pub const RING_CAPACITY: u32 = 13792;

    pub fn from(storage: Rc<RefCell<&'a mut [u8]>>) -> Self {
        Self { storage }
    }
}

impl<'a> Ring<'a> for RewardEventQueue<'a> {
    type Item = RewardEvent;

    fn buffer(&self) -> Rc<RefCell<&'a mut [u8]>> {
        self.storage.clone()
    }
    fn capacity(&self) -> u32 {
        RewardEventQueue::RING_CAPACITY
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum RewardEvent {
    LockedAlloc {
        from: Pubkey,
        total: u64,
        pool: Pubkey,
        vendor: Pubkey,
        mint: Pubkey,
        ts: i64,
    },
    UnlockedAlloc {
        from: Pubkey,
        total: u64,
        pool: Pubkey,
        vendor: Pubkey,
        mint: Pubkey,
        ts: i64,
    },
}

use anchor_lang::{AnchorDeserialize as BorshDeserialize, AnchorSerialize as BorshSerialize};

serum_common::packable!(RewardEvent);
*/
/*
// todo
impl anchor_lang::AccountSerialize for RewardEventQueue {
        fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), ProgramError> {

        }
}

impl anchor_lang::AccountDeserialize for RewardEventQueue {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self, ProgramError> {

        }
}
*/

#[error]
pub enum ErrorCode {
    #[msg("The given reward queue has already been initialized.")]
    RewardQAlreadyInitialized,
    #[msg("The nonce given doesn't derive a valid program address.")]
    InvalidNonce,
    #[msg("Invalid pool mint authority")]
    InvalidPoolMintAuthority,
    #[msg("Member signer doesn't match the derived address.")]
    InvalidMemberSigner,
}
