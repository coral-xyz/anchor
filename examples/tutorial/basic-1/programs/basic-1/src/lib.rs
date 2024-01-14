use anchor_lang::prelude::*;

// macroでProgramのAccountIDを定義している
declare_id!("Dysswo9ycPdcCFKsn2NJGRCB9z7FY1rdiJXBhS6iVQB");

// ProgramLogic部分
#[program]
mod basic_1 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: u64) -> Result<()> {
        let my_account = &mut ctx.accounts.my_account;
        my_account.data = data;
        Ok(())
    }

    pub fn update(ctx: Context<Update>, data: u64) -> Result<()> {
        let my_account = &mut ctx.accounts.my_account;
        my_account.data = data;
        Ok(())
    }
}

// 外部のプログラムと通信するときのデーア構造を定義
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub my_account: Account<'info, MyAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// 外部のプログラムと通信するときのデーア構造を定義
#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub my_account: Account<'info, MyAccount>,
}

// そのデータの所有者を declare_id! で指定されたものに設定する
#[account]
pub struct MyAccount {
    pub data: u64,
}
