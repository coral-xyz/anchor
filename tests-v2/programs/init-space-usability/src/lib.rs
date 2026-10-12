use anchor_lang::prelude::*;

declare_id!("9AbShpmjP5WcQLSBW1NQmczpYVmT2CR2FLFoQdxxk47d");

pub mod limits {
    pub const NAME: usize = 16;
    pub const ITEMS: usize = 4;
    pub const ADD_A: usize = 1;
    pub const ADD_B: usize = 3;
    pub const SUB_N: usize = 10;
}

pub type Scalar = f32;
pub type Coordinate = i32;
pub type Owner = Address;

#[derive(InitSpace)]
pub struct Vector2 {
    pub x: Scalar,
    pub y: Scalar,
}

#[cfg(not(target_os = "solana"))]
#[account(borsh)]
#[derive(InitSpace)]
pub struct Profile {
    pub owner: Owner,
    #[max_len(limits::NAME)]
    pub name: anchor_lang::__idl::string::String,
}

#[cfg(target_os = "solana")]
#[account(borsh)]
#[derive(InitSpace)]
pub struct Profile {
    pub owner: Owner,
    pub name: anchor_lang::FixedVec<u8, { limits::NAME }>,
}

#[cfg(not(target_os = "solana"))]
#[account(borsh)]
#[derive(InitSpace)]
pub struct Nested {
    #[max_len(limits::ITEMS, limits::NAME)]
    pub tags: anchor_lang::__idl::vec::Vec<anchor_lang::__idl::string::String>,
}

#[cfg(target_os = "solana")]
#[account(borsh)]
#[derive(InitSpace)]
pub struct Nested {
    pub tags: anchor_lang::FixedVec<
        anchor_lang::FixedVec<u8, { limits::NAME }>,
        { limits::ITEMS },
    >,
}

#[account(borsh)]
#[derive(InitSpace)]
pub struct Image {
    pub width: Coordinate,
    pub height: Coordinate,
    pub pixels: [u8; limits::ITEMS],
}

#[derive(InitSpace)]
pub struct PointCloud {
    pub points: [Vector2; limits::ITEMS],
}

#[derive(InitSpace)]
pub struct CompoundArrayLengths {
    pub additive: [u16; limits::ADD_A + limits::ADD_B],
    pub subtractive: [u64; limits::SUB_N - 1],
}

#[program]
pub mod init_space_usability {
    use super::*;

    #[discrim = 0]
    pub fn check_constants(_ctx: &mut Context<CheckConstants>) -> Result<()> {
        require_space::<Vector2>(8)?;
        require_space::<Profile>(52)?;
        require_space::<Nested>(84)?;
        require_space::<Image>(12)?;
        require_space::<PointCloud>(32)?;
        require_space::<CompoundArrayLengths>(80)?;
        Ok(())
    }

    #[discrim = 1]
    pub fn init_profile(ctx: &mut Context<InitProfile>) -> Result<()> {
        ctx.accounts.profile.owner = *ctx.accounts.payer.account().address();
        ctx.accounts.profile.name = bounded_string(b"init-space");
        Ok(())
    }

    #[discrim = 2]
    pub fn init_nested(ctx: &mut Context<InitNested>) -> Result<()> {
        #[cfg(not(target_os = "solana"))]
        let mut tags = anchor_lang::__idl::vec::Vec::new();
        #[cfg(target_os = "solana")]
        let mut tags = anchor_lang::FixedVec::<
            anchor_lang::FixedVec<u8, { limits::NAME }>,
            { limits::ITEMS },
        >::new();
        tags.push(bounded_string(b"zero"));
        tags.push(bounded_string(b"one"));
        tags.push(bounded_string(b"two"));
        tags.push(bounded_string(b"three"));
        ctx.accounts.nested.tags = tags;
        Ok(())
    }

    #[discrim = 3]
    pub fn init_image(ctx: &mut Context<InitImage>) -> Result<()> {
        ctx.accounts.image.width = 640;
        ctx.accounts.image.height = 480;
        ctx.accounts.image.pixels = [1, 2, 3, 4];
        Ok(())
    }
}

#[cfg(not(target_os = "solana"))]
fn bounded_string(value: &[u8]) -> anchor_lang::__idl::string::String {
    anchor_lang::__idl::string::String::from(core::str::from_utf8(value).unwrap())
}

#[cfg(target_os = "solana")]
fn bounded_string(value: &[u8]) -> anchor_lang::FixedVec<u8, { limits::NAME }> {
    let mut string = anchor_lang::FixedVec::new();
    string.extend_from_slice(value);
    string
}

fn require_space<T: Space>(expected: usize) -> Result<()> {
    if T::INIT_SPACE != expected {
        return Err(ProgramError::InvalidAccountData.into());
    }
    Ok(())
}

#[derive(Accounts)]
pub struct CheckConstants {}

#[derive(Accounts)]
pub struct InitProfile {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer, space = 8 + Profile::INIT_SPACE, seeds = [b"profile"], bump)]
    pub profile: BorshAccount<Profile>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct InitNested {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer, space = 8 + Nested::INIT_SPACE, seeds = [b"nested"], bump)]
    pub nested: BorshAccount<Nested>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct InitImage {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer, space = 8 + Image::INIT_SPACE, seeds = [b"image"], bump)]
    pub image: BorshAccount<Image>,
    pub system_program: Program<System>,
}
