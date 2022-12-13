use crate::account::*;
use anchor_lang::accounts::loader::Loader;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct TestTokenSeedsInit<'info> {
    #[account(
        init,
        seeds = [b"my-mint-seed".as_ref()],
        bump,
        payer = authority,
        mint::decimals = 6,
        mint::authority = authority,
    )]
    pub mint: Option<Account<'info, Mint>>,
    #[account(
        init,
        seeds = [b"my-token-seed".as_ref()],
        bump,
        payer = authority,
        token::mint = mint,
        token::authority = authority,
    )]
    pub my_pda: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// CHECK:
    pub authority: Option<AccountInfo<'info>>,
    pub system_program: Option<Program<'info, System>>,
    pub token_program: Option<Program<'info, Token>>,
}

#[derive(Accounts)]
pub struct TestInitAssociatedToken<'info> {
    #[account(
        init,
        associated_token::mint = mint,
        payer = payer,
        associated_token::authority = payer,
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
    pub token_program: Option<Program<'info, Token>>,
    pub associated_token_program: Option<Program<'info, AssociatedToken>>,
}

#[derive(Accounts)]
pub struct TestValidateAssociatedToken<'info> {
    #[account(
        associated_token::mint = mint,
        associated_token::authority = wallet,
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    /// CHECK:
    pub wallet: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct TestInstructionConstraint<'info> {
    #[account(
        seeds = [b"my-seed", my_account.as_ref().unwrap().key.as_ref()],
        bump = nonce,
    )]
    /// CHECK:
    pub my_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    pub my_account: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
#[instruction(domain: String, seed: Vec<u8>, bump: u8)]
pub struct TestPdaInit<'info> {
    #[account(
        init,
        seeds = [b"my-seed", domain.as_bytes(), foo.as_ref().unwrap().key.as_ref(), &seed],
        bump,
        payer = my_payer,
        space = DataU16::LEN + 8
    )]
    pub my_pda: Option<Account<'info, DataU16>>,
    #[account(mut)]
    pub my_payer: Option<Signer<'info>>,
    /// CHECK:
    pub foo: Option<AccountInfo<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestPdaInitZeroCopy<'info> {
    #[account(
        init,
        seeds = [b"my-seed".as_ref()],
        bump,
        payer = my_payer,
        space = DataZeroCopy::LEN + 8
    )]
    pub my_pda: Option<AccountLoader<'info, DataZeroCopy>>,
    #[account(mut)]
    pub my_payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestPdaMutZeroCopy<'info> {
    #[account(
        mut,
        seeds = [b"my-seed".as_ref()],
        bump = my_pda.load()?.bump,
    )]
    pub my_pda: Option<AccountLoader<'info, DataZeroCopy>>,
    /// CHECK:
    pub my_payer: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub data: Option<Account<'info, Data>>,
}

#[derive(Accounts)]
pub struct InitializeSkipRentExempt<'info> {
    #[account(zero, rent_exempt = skip)]
    pub data: Option<Account<'info, Data>>,
}

#[derive(Accounts)]
pub struct InitializeNoRentExempt<'info> {
    /// CHECK:
    pub data: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestOwner<'info> {
    #[account(owner = *misc.key)]
    /// CHECK:
    pub data: Option<AccountInfo<'info>>,
    /// CHECK:
    pub misc: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TestExecutable<'info> {
    #[account(executable)]
    /// CHECK:
    pub program: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestClose<'info> {
    #[account(mut, close = sol_dest)]
    pub data: Option<Account<'info, Data>>,
    /// CHECK:
    sol_dest: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestCloseTwice<'info> {
    #[account(mut, close = sol_dest)]
    pub data: Option<Account<'info, Data>>,
    /// CHECK:
    pub sol_dest: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestCloseMut<'info> {
    #[account(mut)]
    pub data: Option<Account<'info, Data>>,
    /// CHECK:
    pub sol_dest: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestU16<'info> {
    #[account(zero)]
    pub my_account: Option<Account<'info, DataU16>>,
}

#[derive(Accounts)]
pub struct TestI16<'info> {
    #[account(zero)]
    pub data: Option<Account<'info, DataI16>>,
}

#[derive(Accounts)]
pub struct TestSimulate {}

#[derive(Accounts)]
pub struct TestI8<'info> {
    #[account(zero)]
    pub data: Option<Account<'info, DataI8>>,
}

#[derive(Accounts)]
pub struct TestCompositePayer<'info> {
    pub composite: TestInit<'info>,
    #[account(init, payer = payer.as_ref().unwrap(), space = Data::LEN + 8)]
    pub data: Option<Account<'info, Data>>,
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestInit<'info> {
    #[account(init, payer = payer, space = DataI8::LEN + 8)]
    pub data: Option<Account<'info, DataI8>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestInitZeroCopy<'info> {
    #[account(init, payer = payer, space = DataZeroCopy::LEN + 8)]
    pub data: Option<Loader<'info, DataZeroCopy>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestInitMint<'info> {
    #[account(init, mint::decimals = 6, mint::authority = payer, mint::freeze_authority = payer, payer = payer, )]
    pub mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
    pub token_program: Option<Program<'info, Token>>,
}

#[derive(Accounts)]
pub struct TestInitToken<'info> {
    #[account(init, token::mint = mint, token::authority = payer, payer = payer, )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
    pub token_program: Option<Program<'info, Token>>,
}

#[derive(Accounts)]
pub struct TestFetchAll<'info> {
    #[account(init, payer = authority, space = DataWithFilter::LEN + 8)]
    pub data: Option<Account<'info, DataWithFilter>>,
    #[account(mut)]
    pub authority: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestInitWithEmptySeeds<'info> {
    #[account(init, seeds = [], bump, payer = authority, space = Data::LEN + 8)]
    pub pda: Option<Account<'info, Data>>,
    #[account(mut)]
    pub authority: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestEmptySeedsConstraint<'info> {
    #[account(seeds = [], bump)]
    /// CHECK:
    pub pda: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct InitWithSpace<'info> {
    #[account(init, payer = payer, space = DataU16::LEN + 8)]
    pub data: Option<Account<'info, DataU16>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestInitIfNeeded<'info> {
    // intentionally using more space (+500) to check whether space is checked when using init_if_needed
    #[account(init_if_needed, payer = payer, space = DataU16::LEN + 8 + 500)]
    pub data: Option<Account<'info, DataU16>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestInitIfNeededChecksOwner<'info> {
    #[account(init_if_needed, payer = payer, space = 100, owner = *owner.key, seeds = [b"hello"], bump)]
    /// CHECK:
    pub data: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
    /// CHECK:
    pub owner: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(seed_data: String)]
pub struct TestInitIfNeededChecksSeeds<'info> {
    #[account(init_if_needed, payer = payer, space = 100, seeds = [seed_data.as_bytes()], bump)]
    /// CHECK:
    pub data: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct TestInitMintIfNeeded<'info> {
    #[account(init_if_needed, mint::decimals = decimals, mint::authority = mint_authority, mint::freeze_authority = freeze_authority, payer = payer)]
    pub mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
    pub token_program: Option<Program<'info, Token>>,
    /// CHECK:
    pub mint_authority: Option<AccountInfo<'info>>,
    /// CHECK:
    pub freeze_authority: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestInitTokenIfNeeded<'info> {
    #[account(init_if_needed, token::mint = mint, token::authority = authority, payer = payer, )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
    pub token_program: Option<Program<'info, Token>>,
    /// CHECK:
    pub authority: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestInitAssociatedTokenIfNeeded<'info> {
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub payer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
    pub token_program: Option<Program<'info, Token>>,
    pub associated_token_program: Option<Program<'info, AssociatedToken>>,
    /// CHECK:
    pub authority: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestMultidimensionalArray<'info> {
    #[account(zero)]
    pub data: Option<Account<'info, DataMultidimensionalArray>>,
}

#[derive(Accounts)]
pub struct TestConstArraySize<'info> {
    #[account(zero)]
    pub data: Option<Account<'info, DataConstArraySize>>,
}

#[derive(Accounts)]
pub struct TestConstIxDataSize<'info> {
    #[account(zero)]
    pub data: Option<Account<'info, DataConstArraySize>>,
}

#[derive(Accounts)]
pub struct TestMultidimensionalArrayConstSizes<'info> {
    #[account(zero)]
    pub data: Option<Account<'info, DataMultidimensionalArrayConstSizes>>,
}

#[derive(Accounts)]
pub struct NoRentExempt<'info> {
    /// CHECK:
    pub data: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct EnforceRentExempt<'info> {
    #[account(rent_exempt = enforce)]
    /// CHECK:
    pub data: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct InitDecreaseLamports<'info> {
    #[account(init, payer = user, space = 1000)]
    /// CHECK:
    pub data: Option<AccountInfo<'info>>,
    #[account(mut)]
    pub user: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct InitIfNeededChecksRentExemption<'info> {
    #[account(init_if_needed, payer = user, space = 1000)]
    /// CHECK:
    pub data: Option<AccountInfo<'info>>,
    #[account(mut)]
    pub user: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
#[instruction(bump: u8, second_bump: u8)]
pub struct TestProgramIdConstraint<'info> {
    // not a real associated token account
    // just deriving like this for testing purposes
    #[account(seeds = [b"seed"], bump = bump, seeds::program = anchor_spl::associated_token::ID)]
    /// CHECK:
    first: Option<AccountInfo<'info>>,

    #[account(seeds = [b"seed"], bump = second_bump, seeds::program = crate::ID)]
    /// CHECK:
    second: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestProgramIdConstraintUsingFindPda<'info> {
    // not a real associated token account
    // just deriving like this for testing purposes
    #[account(seeds = [b"seed"], bump, seeds::program = anchor_spl::associated_token::ID)]
    /// CHECK:
    first: Option<AccountInfo<'info>>,

    #[account(seeds = [b"seed"], bump, seeds::program = crate::ID)]
    /// CHECK:
    second: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestUnsafeFieldSafetyErrors<'info> {
    #[doc = "test"]
    /// CHECK:
    pub data: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    /// CHECK:
    pub data_two: Option<UncheckedAccount<'info>>,
    #[account(
        seeds = [b"my-seed", signer.as_ref().unwrap().key.as_ref()],
        bump
    )]
    /// CHECK:
    pub data_three: Option<UncheckedAccount<'info>>,
    /// CHECK:
    pub data_four: Option<UncheckedAccount<'info>>,
    pub signer: Option<Signer<'info>>,
    pub system_program: Option<Program<'info, System>>,
}

#[derive(Accounts)]
pub struct TestConstraintToken<'info> {
    #[account(
        token::mint = mint,
        token::authority = payer
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    pub payer: Option<Signer<'info>>,
}

#[derive(Accounts)]
pub struct TestAuthorityConstraint<'info> {
    #[account(
        token::mint = mint,
        token::authority = fake_authority
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    pub fake_authority: Option<AccountInfo<'info>>,
}
#[derive(Accounts)]
pub struct TestOnlyAuthorityConstraint<'info> {
    #[account(
        token::authority = payer
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    pub payer: Option<Signer<'info>>,
}
#[derive(Accounts)]
pub struct TestOnlyMintConstraint<'info> {
    #[account(
        token::mint = mint,
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct TestMintConstraint<'info> {
    #[account(
        mint::decimals = decimals,
        mint::authority = mint_authority,
        mint::freeze_authority = freeze_authority
    )]
    pub mint: Option<Account<'info, Mint>>,
    pub mint_authority: Option<AccountInfo<'info>>,
    pub freeze_authority: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct TestMintOnlyDecimalsConstraint<'info> {
    #[account(
        mint::decimals = decimals,
    )]
    pub mint: Option<Account<'info, Mint>>,
}

#[derive(Accounts)]
pub struct TestMintAuthorityConstraint<'info> {
    #[account(
        mint::authority = mint_authority,
        mint::freeze_authority = freeze_authority
    )]
    pub mint: Option<Account<'info, Mint>>,
    pub mint_authority: Option<AccountInfo<'info>>,
    pub freeze_authority: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestMintOneAuthorityConstraint<'info> {
    #[account(
        mint::authority = mint_authority,
    )]
    pub mint: Option<Account<'info, Mint>>,
    pub mint_authority: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct TestMintMissMintAuthConstraint<'info> {
    #[account(
        mint::decimals = decimals,
        mint::freeze_authority = freeze_authority,
    )]
    pub mint: Option<Account<'info, Mint>>,
    pub freeze_authority: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct TestAssociatedToken<'info> {
    #[account(
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token: Option<Account<'info, TokenAccount>>,
    pub mint: Option<Account<'info, Mint>>,
    pub authority: Option<AccountInfo<'info>>,
}
