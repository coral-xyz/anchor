#![feature(proc_macro_hygiene)]

use anchor::prelude::*;

// Define the program's instruction handlers.

#[program]
mod basic_2 {
    use super::*;

    pub fn create_author(
        ctx: Context<CreateAuthor>,
        authority: Pubkey,
        name: String,
    ) -> ProgramResult {
        let author = &mut ctx.accounts.author;
        author.authority = authority;
        author.name = name;
        Ok(())
    }

    pub fn update_author(ctx: Context<UpdateAuthor>, name: String) -> ProgramResult {
        let author = &mut ctx.accounts.author;
        author.name = name;
        Ok(())
    }

    pub fn create_book(ctx: Context<CreateBook>, title: String, pages: Vec<Page>) -> ProgramResult {
        let book = &mut ctx.accounts.book;
        book.author = *ctx.accounts.author.to_account_info().key;
        book.title = title;
        book.pages = pages;
        Ok(())
    }

    pub fn update_book(
        ctx: Context<UpdateBook>,
        title: Option<String>,
        pages: Option<Vec<Page>>,
    ) -> ProgramResult {
        let book = &mut ctx.accounts.book;
        if let Some(title) = title {
            book.title = title;
        }
        if let Some(pages) = pages {
            book.pages = pages;
        }
        Ok(())
    }

    pub fn composite_update(
        ctx: Context<CompositeUpdate>,
        author_name: String,
        book_title: String,
    ) -> ProgramResult {
        let update_author_ctx = Context::new(ctx.program_id, &mut ctx.accounts.update_author, &[]);
        let update_book_ctx = Context::new(ctx.program_id, &mut ctx.accounts.update_book, &[]);

        update_author(update_author_ctx, author_name)?;
        update_book(update_book_ctx, Some(book_title), None)?;

        Ok(())
    }
}

// Define the validated accounts for each handler.

#[derive(Accounts)]
pub struct CreateAuthor<'info> {
    #[account(init)]
    pub author: ProgramAccount<'info, Author>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateAuthor<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut, "&author.authority == authority.key")]
    pub author: ProgramAccount<'info, Author>,
}

#[derive(Accounts)]
pub struct CreateBook<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account("&author.authority == authority.key")]
    pub author: ProgramAccount<'info, Author>,
    #[account(init)]
    pub book: ProgramAccount<'info, Book>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateBook<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account("&author.authority == authority.key")]
    pub author: ProgramAccount<'info, Author>,
    #[account(mut, belongs_to = author)]
    pub book: ProgramAccount<'info, Book>,
}

#[derive(Accounts)]
pub struct CompositeUpdate<'info> {
    update_author: UpdateAuthor<'info>,
    update_book: UpdateBook<'info>,
}

// Define the program owned accounts.

#[account]
pub struct Author {
    pub authority: Pubkey,
    pub name: String,
}

#[account]
pub struct Book {
    pub author: Pubkey,
    pub title: String,
    pub pages: Vec<Page>,
}

// Define custom types.

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Page {
    pub content: String,
    pub footnote: String,
}

// Define any auxiliary access control checks.

fn not_zero(authority: Pubkey) -> ProgramResult {
    if authority == Pubkey::new_from_array([0; 32]) {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok(())
}
