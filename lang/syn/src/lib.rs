use crate::parser::tts_to_string;
use codegen::accounts as accounts_codegen;
use codegen::program as program_codegen;
use parser::accounts as accounts_parser;
use parser::program as program_parser;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use std::collections::HashMap;
use std::ops::Deref;
use syn::ext::IdentExt;
use syn::parse::{Error as ParseError, Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    Expr, Generics, Ident, ItemEnum, ItemFn, ItemMod, ItemStruct, LitInt, PatType, Token, Type,
    TypePath,
};

pub mod codegen;
#[cfg(feature = "hash")]
pub mod hash;
#[cfg(not(feature = "hash"))]
pub(crate) mod hash;
pub mod idl;
pub mod parser;

#[derive(Debug)]
pub struct Program {
    pub ixs: Vec<Ix>,
    pub name: Ident,
    pub docs: Option<Vec<String>>,
    pub program_mod: ItemMod,
    pub fallback_fn: Option<FallbackFn>,
}

impl Parse for Program {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let program_mod = <ItemMod as Parse>::parse(input)?;
        program_parser::parse(program_mod)
    }
}

impl From<&Program> for TokenStream {
    fn from(program: &Program) -> Self {
        program_codegen::generate(program)
    }
}

impl ToTokens for Program {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend::<TokenStream>(self.into());
    }
}

#[derive(Debug)]
pub struct Ix {
    pub raw_method: ItemFn,
    pub ident: Ident,
    pub docs: Option<Vec<String>>,
    pub args: Vec<IxArg>,
    pub returns: IxReturn,
    // The ident for the struct deriving Accounts.
    pub anchor_ident: Ident,
}

#[derive(Debug)]
pub struct IxArg {
    pub name: Ident,
    pub docs: Option<Vec<String>>,
    pub raw_arg: PatType,
}

#[derive(Debug)]
pub struct IxReturn {
    pub ty: Type,
}

#[derive(Debug)]
pub struct FallbackFn {
    raw_method: ItemFn,
}

#[derive(Debug)]
pub struct AccountsStruct {
    // Name of the accounts struct.
    pub ident: Ident,
    // Generics + lifetimes on the accounts struct.
    pub generics: Generics,
    // Fields on the accounts struct.
    pub fields: Vec<AccountField>,
    // Instruction data api expression.
    instruction_api: Option<Punctuated<Expr, Comma>>,
}

impl Parse for AccountsStruct {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let strct = <ItemStruct as Parse>::parse(input)?;
        accounts_parser::parse(&strct)
    }
}

impl From<&AccountsStruct> for TokenStream {
    fn from(accounts: &AccountsStruct) -> Self {
        accounts_codegen::generate(accounts)
    }
}

impl ToTokens for AccountsStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend::<TokenStream>(self.into());
    }
}

impl AccountsStruct {
    pub fn new(
        strct: ItemStruct,
        fields: Vec<AccountField>,
        instruction_api: Option<Punctuated<Expr, Comma>>,
    ) -> Self {
        let ident = strct.ident.clone();
        let generics = strct.generics;
        Self {
            ident,
            generics,
            fields,
            instruction_api,
        }
    }

    // Return value maps instruction name to type.
    // E.g. if we have `#[instruction(data: u64)]` then returns
    // { "data": "u64"}.
    pub fn instruction_args(&self) -> Option<HashMap<String, String>> {
        self.instruction_api.as_ref().map(|instruction_api| {
            instruction_api
                .iter()
                .map(|expr| {
                    let arg = parser::tts_to_string(expr);
                    let components: Vec<&str> = arg.split(" : ").collect();
                    assert!(components.len() == 2);
                    (components[0].to_string(), components[1].to_string())
                })
                .collect()
        })
    }

    pub fn field_names(&self) -> Vec<String> {
        self.fields
            .iter()
            .map(|field| field.ident().to_string())
            .collect()
    }

    pub fn has_optional(&self) -> bool {
        for field in &self.fields {
            if let AccountField::Field(field) = field {
                if field.is_optional {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_field_optional<T: quote::ToTokens>(&self, field: &T) -> bool {
        let matching_field = self
            .fields
            .iter()
            .find(|f| *f.ident() == tts_to_string(field));
        if let Some(matching_field) = matching_field {
            matching_field.is_optional()
        } else {
            false
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum AccountField {
    Field(Field),
    CompositeField(CompositeField),
}

impl AccountField {
    fn ident(&self) -> &Ident {
        match self {
            AccountField::Field(field) => &field.ident,
            AccountField::CompositeField(c_field) => &c_field.ident,
        }
    }

    fn is_optional(&self) -> bool {
        match self {
            AccountField::Field(field) => field.is_optional,
            AccountField::CompositeField(_) => false,
        }
    }

    pub fn ty_name(&self) -> Option<String> {
        let qualified_ty_name = match self {
            AccountField::Field(field) => match &field.ty {
                Ty::Account(account) => Some(parser::tts_to_string(&account.account_type_path)),
                _ => None,
            },
            AccountField::CompositeField(field) => Some(field.symbol.clone()),
        };

        qualified_ty_name.map(|name| match name.rsplit_once(" :: ") {
            Some((_prefix, suffix)) => suffix.to_string(),
            None => name,
        })
    }
}

#[derive(Debug)]
pub struct Field {
    pub ident: Ident,
    pub constraints: ConstraintGroup,
    pub ty: Ty,
    pub is_optional: bool,
    /// IDL Doc comment
    pub docs: Option<Vec<String>>,
}

impl Field {
    pub fn typed_ident(&self) -> proc_macro2::TokenStream {
        let name = &self.ident;
        let ty_decl = self.ty_decl(false);
        quote! {
            #name: #ty_decl
        }
    }

    pub fn ty_decl(&self, ignore_option: bool) -> proc_macro2::TokenStream {
        let account_ty = self.account_ty();
        let container_ty = self.container_ty();
        let inner_ty = match &self.ty {
            Ty::AccountInfo => quote! {
                AccountInfo
            },
            Ty::UncheckedAccount => quote! {
                UncheckedAccount
            },
            Ty::Signer => quote! {
                Signer
            },
            Ty::ProgramData => quote! {
                ProgramData
            },
            Ty::SystemAccount => quote! {
                SystemAccount
            },
            Ty::Account(AccountTy { boxed, .. })
            | Ty::InterfaceAccount(InterfaceAccountTy { boxed, .. }) => {
                if *boxed {
                    quote! {
                        Box<#container_ty<#account_ty>>
                    }
                } else {
                    quote! {
                        #container_ty<#account_ty>
                    }
                }
            }
            Ty::Sysvar(ty) => {
                let account = match ty {
                    SysvarTy::Clock => quote! {Clock},
                    SysvarTy::Rent => quote! {Rent},
                    SysvarTy::EpochSchedule => quote! {EpochSchedule},
                    SysvarTy::Fees => quote! {Fees},
                    SysvarTy::RecentBlockhashes => quote! {RecentBlockhashes},
                    SysvarTy::SlotHashes => quote! {SlotHashes},
                    SysvarTy::SlotHistory => quote! {SlotHistory},
                    SysvarTy::StakeHistory => quote! {StakeHistory},
                    SysvarTy::Instructions => quote! {Instructions},
                    SysvarTy::Rewards => quote! {Rewards},
                };
                quote! {
                    Sysvar<#account>
                }
            }
            _ => quote! {
                #container_ty<#account_ty>
            },
        };
        if self.is_optional && !ignore_option {
            quote! {
                Option<#inner_ty>
            }
        } else {
            quote! {
                #inner_ty
            }
        }
    }

    // Ignores optional accounts. Optional account checks and handing should be done prior to this
    // function being called.
    pub fn from_account_info(
        &self,
        kind: Option<&InitKind>,
        checked: bool,
    ) -> proc_macro2::TokenStream {
        let field = &self.ident;
        let field_str = field.to_string();
        let container_ty = self.container_ty();
        let owner_addr = match &kind {
            None => quote! { __program_id },
            Some(InitKind::Program { .. }) => quote! {
                __program_id
            },
            _ => quote! {
                &anchor_spl::token::ID
            },
        };
        match &self.ty {
            Ty::AccountInfo => quote! { #field.to_account_info() },
            Ty::UncheckedAccount => {
                quote! { UncheckedAccount::try_from(#field.to_account_info()) }
            }
            Ty::Account(AccountTy { boxed, .. })
            | Ty::InterfaceAccount(InterfaceAccountTy { boxed, .. }) => {
                let stream = if checked {
                    quote! {
                        match #container_ty::try_from(&#field) {
                            Ok(val) => val,
                            Err(e) => return Err(e.with_account_name(#field_str))
                        }
                    }
                } else {
                    quote! {
                        match #container_ty::try_from_unchecked(&#field) {
                            Ok(val) => val,
                            Err(e) => return Err(e.with_account_name(#field_str))
                        }
                    }
                };
                if *boxed {
                    quote! {
                        Box::new(#stream)
                    }
                } else {
                    stream
                }
            }
            Ty::AccountLoader(_) => {
                if checked {
                    quote! {
                        match #container_ty::try_from(&#field) {
                            Ok(val) => val,
                            Err(e) => return Err(e.with_account_name(#field_str))
                        }
                    }
                } else {
                    quote! {
                        match #container_ty::try_from_unchecked(#owner_addr, &#field) {
                            Ok(val) => val,
                            Err(e) => return Err(e.with_account_name(#field_str))
                        }
                    }
                }
            }
            _ => {
                if checked {
                    quote! {
                        match #container_ty::try_from(#owner_addr, &#field) {
                            Ok(val) => val,
                            Err(e) => return Err(e.with_account_name(#field_str))
                        }
                    }
                } else {
                    quote! {
                        match #container_ty::try_from_unchecked(#owner_addr, &#field) {
                            Ok(val) => val,
                            Err(e) => return Err(e.with_account_name(#field_str))
                        }
                    }
                }
            }
        }
    }

    pub fn container_ty(&self) -> proc_macro2::TokenStream {
        match &self.ty {
            Ty::Account(_) => quote! {
                anchor_lang::accounts::account::Account
            },
            Ty::AccountLoader(_) => quote! {
                anchor_lang::accounts::account_loader::AccountLoader
            },
            Ty::Sysvar(_) => quote! { anchor_lang::accounts::sysvar::Sysvar },
            Ty::Program(_) => quote! { anchor_lang::accounts::program::Program },
            Ty::Interface(_) => quote! { anchor_lang::accounts::interface::Interface },
            Ty::InterfaceAccount(_) => {
                quote! { anchor_lang::accounts::interface_account::InterfaceAccount }
            }
            Ty::AccountInfo => quote! {},
            Ty::UncheckedAccount => quote! {},
            Ty::Signer => quote! {},
            Ty::SystemAccount => quote! {},
            Ty::ProgramData => quote! {},
        }
    }

    // Returns the inner account struct type.
    pub fn account_ty(&self) -> proc_macro2::TokenStream {
        match &self.ty {
            Ty::AccountInfo => quote! {
                AccountInfo
            },
            Ty::UncheckedAccount => quote! {
                UncheckedAccount
            },
            Ty::Signer => quote! {
                Signer
            },
            Ty::SystemAccount => quote! {
                SystemAccount
            },
            Ty::ProgramData => quote! {
                ProgramData
            },
            Ty::Account(ty) => {
                let ident = &ty.account_type_path;
                quote! {
                    #ident
                }
            }
            Ty::InterfaceAccount(ty) => {
                let ident = &ty.account_type_path;
                quote! {
                    #ident
                }
            }
            Ty::AccountLoader(ty) => {
                let ident = &ty.account_type_path;
                quote! {
                    #ident
                }
            }
            Ty::Sysvar(ty) => match ty {
                SysvarTy::Clock => quote! {Clock},
                SysvarTy::Rent => quote! {Rent},
                SysvarTy::EpochSchedule => quote! {EpochSchedule},
                SysvarTy::Fees => quote! {Fees},
                SysvarTy::RecentBlockhashes => quote! {RecentBlockhashes},
                SysvarTy::SlotHashes => quote! {SlotHashes},
                SysvarTy::SlotHistory => quote! {SlotHistory},
                SysvarTy::StakeHistory => quote! {StakeHistory},
                SysvarTy::Instructions => quote! {Instructions},
                SysvarTy::Rewards => quote! {Rewards},
            },
            Ty::Program(ty) => {
                let program = &ty.account_type_path;
                quote! {
                    #program
                }
            }
            Ty::Interface(ty) => {
                let program = &ty.account_type_path;
                quote! {
                    #program
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct CompositeField {
    pub ident: Ident,
    pub constraints: ConstraintGroup,
    pub symbol: String,
    pub raw_field: syn::Field,
    /// IDL Doc comment
    pub docs: Option<Vec<String>>,
}

// A type of an account field.
#[derive(Debug, PartialEq, Eq)]
pub enum Ty {
    AccountInfo,
    UncheckedAccount,
    AccountLoader(AccountLoaderTy),
    Sysvar(SysvarTy),
    Account(AccountTy),
    Program(ProgramTy),
    Interface(InterfaceTy),
    InterfaceAccount(InterfaceAccountTy),
    Signer,
    SystemAccount,
    ProgramData,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SysvarTy {
    Clock,
    Rent,
    EpochSchedule,
    Fees,
    RecentBlockhashes,
    SlotHashes,
    SlotHistory,
    StakeHistory,
    Instructions,
    Rewards,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AccountLoaderTy {
    // The struct type of the account.
    pub account_type_path: TypePath,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AccountTy {
    // The struct type of the account.
    pub account_type_path: TypePath,
    // True if the account has been boxed via `Box<T>`.
    pub boxed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterfaceAccountTy {
    // The struct type of the account.
    pub account_type_path: TypePath,
    // True if the account has been boxed via `Box<T>`.
    pub boxed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProgramTy {
    // The struct type of the account.
    pub account_type_path: TypePath,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterfaceTy {
    // The struct type of the account.
    pub account_type_path: TypePath,
}

#[derive(Debug)]
pub struct Error {
    pub name: String,
    pub raw_enum: ItemEnum,
    pub ident: Ident,
    pub codes: Vec<ErrorCode>,
    pub args: Option<ErrorArgs>,
}

#[derive(Debug)]
pub struct ErrorArgs {
    pub offset: LitInt,
}

impl Parse for ErrorArgs {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let offset_span = stream.span();
        let offset = stream.call(Ident::parse_any)?;
        if offset.to_string().as_str() != "offset" {
            return Err(ParseError::new(offset_span, "expected keyword offset"));
        }
        stream.parse::<Token![=]>()?;
        Ok(ErrorArgs {
            offset: stream.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct ErrorCode {
    pub id: u32,
    pub ident: Ident,
    pub msg: Option<String>,
}

// All well formed constraints on a single `Accounts` field.
#[derive(Debug, Default, Clone)]
pub struct ConstraintGroup {
    pub init: Option<ConstraintInitGroup>,
    pub zeroed: Option<ConstraintZeroed>,
    pub mutable: Option<ConstraintMut>,
    pub signer: Option<ConstraintSigner>,
    pub owner: Option<ConstraintOwner>,
    pub rent_exempt: Option<ConstraintRentExempt>,
    pub seeds: Option<ConstraintSeedsGroup>,
    pub executable: Option<ConstraintExecutable>,
    pub has_one: Vec<ConstraintHasOne>,
    pub raw: Vec<ConstraintRaw>,
    pub close: Option<ConstraintClose>,
    pub address: Option<ConstraintAddress>,
    pub associated_token: Option<ConstraintAssociatedToken>,
    pub token_account: Option<ConstraintTokenAccountGroup>,
    pub mint: Option<ConstraintTokenMintGroup>,
    pub realloc: Option<ConstraintReallocGroup>,
}

impl ConstraintGroup {
    pub fn is_zeroed(&self) -> bool {
        self.zeroed.is_some()
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable.is_some()
    }

    pub fn is_signer(&self) -> bool {
        self.signer.is_some()
    }

    pub fn is_close(&self) -> bool {
        self.close.is_some()
    }
}

// A single account constraint *after* merging all tokens into a well formed
// constraint. Some constraints like "seeds" are defined by multiple
// tokens, so a merging phase is required.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Constraint {
    Init(ConstraintInitGroup),
    Zeroed(ConstraintZeroed),
    Mut(ConstraintMut),
    Signer(ConstraintSigner),
    HasOne(ConstraintHasOne),
    Raw(ConstraintRaw),
    Owner(ConstraintOwner),
    RentExempt(ConstraintRentExempt),
    Seeds(ConstraintSeedsGroup),
    AssociatedToken(ConstraintAssociatedToken),
    Executable(ConstraintExecutable),
    Close(ConstraintClose),
    Address(ConstraintAddress),
    TokenAccount(ConstraintTokenAccountGroup),
    Mint(ConstraintTokenMintGroup),
    Realloc(ConstraintReallocGroup),
}

// Constraint token is a single keyword in a `#[account(<TOKEN>)]` attribute.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum ConstraintToken {
    Init(Context<ConstraintInit>),
    Zeroed(Context<ConstraintZeroed>),
    Mut(Context<ConstraintMut>),
    Signer(Context<ConstraintSigner>),
    HasOne(Context<ConstraintHasOne>),
    Raw(Context<ConstraintRaw>),
    Owner(Context<ConstraintOwner>),
    RentExempt(Context<ConstraintRentExempt>),
    Seeds(Context<ConstraintSeeds>),
    Executable(Context<ConstraintExecutable>),
    Close(Context<ConstraintClose>),
    Payer(Context<ConstraintPayer>),
    Space(Context<ConstraintSpace>),
    Address(Context<ConstraintAddress>),
    TokenMint(Context<ConstraintTokenMint>),
    TokenAuthority(Context<ConstraintTokenAuthority>),
    TokenTokenProgram(Context<ConstraintTokenProgram>),
    AssociatedTokenMint(Context<ConstraintTokenMint>),
    AssociatedTokenAuthority(Context<ConstraintTokenAuthority>),
    AssociatedTokenTokenProgram(Context<ConstraintTokenProgram>),
    MintAuthority(Context<ConstraintMintAuthority>),
    MintFreezeAuthority(Context<ConstraintMintFreezeAuthority>),
    MintDecimals(Context<ConstraintMintDecimals>),
    MintTokenProgram(Context<ConstraintTokenProgram>),
    Bump(Context<ConstraintTokenBump>),
    ProgramSeed(Context<ConstraintProgramSeed>),
    Realloc(Context<ConstraintRealloc>),
    ReallocPayer(Context<ConstraintReallocPayer>),
    ReallocZero(Context<ConstraintReallocZero>),
}

impl Parse for ConstraintToken {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        accounts_parser::constraints::parse_token(stream)
    }
}

#[derive(Debug, Clone)]
pub struct ConstraintInit {
    pub if_needed: bool,
}

#[derive(Debug, Clone)]
pub struct ConstraintInitIfNeeded {}

#[derive(Debug, Clone)]
pub struct ConstraintZeroed {}

#[derive(Debug, Clone)]
pub struct ConstraintMut {
    pub error: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintReallocGroup {
    pub payer: Expr,
    pub space: Expr,
    pub zero: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintRealloc {
    pub space: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintReallocPayer {
    pub target: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintReallocZero {
    pub zero: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintSigner {
    pub error: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintHasOne {
    pub join_target: Expr,
    pub error: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintRaw {
    pub raw: Expr,
    pub error: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintOwner {
    pub owner_address: Expr,
    pub error: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintAddress {
    pub address: Expr,
    pub error: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum ConstraintRentExempt {
    Enforce,
    Skip,
}

#[derive(Debug, Clone)]
pub struct ConstraintInitGroup {
    pub if_needed: bool,
    pub seeds: Option<ConstraintSeedsGroup>,
    pub payer: Expr,
    pub space: Option<Expr>,
    pub kind: InitKind,
}

#[derive(Debug, Clone)]
pub struct ConstraintSeedsGroup {
    pub is_init: bool,
    pub seeds: Punctuated<Expr, Token![,]>,
    pub bump: Option<Expr>,         // None => bump was given without a target.
    pub program_seed: Option<Expr>, // None => use the current program's program_id.
}

#[derive(Debug, Clone)]
pub struct ConstraintSeeds {
    pub seeds: Punctuated<Expr, Token![,]>,
}

#[derive(Debug, Clone)]
pub struct ConstraintExecutable {}

#[derive(Debug, Clone)]
pub struct ConstraintPayer {
    pub target: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintSpace {
    pub space: Expr,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum InitKind {
    Program {
        owner: Option<Expr>,
    },
    Interface {
        owner: Option<Expr>,
    },
    // Owner for token and mint represents the authority. Not to be confused
    // with the owner of the AccountInfo.
    Token {
        owner: Expr,
        mint: Expr,
        token_program: Option<Expr>,
    },
    AssociatedToken {
        owner: Expr,
        mint: Expr,
        token_program: Option<Expr>,
    },
    Mint {
        owner: Expr,
        freeze_authority: Option<Expr>,
        decimals: Expr,
        token_program: Option<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct ConstraintClose {
    pub sol_dest: Ident,
}

#[derive(Debug, Clone)]
pub struct ConstraintTokenMint {
    pub mint: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintTokenAuthority {
    pub auth: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintTokenProgram {
    token_program: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintMintAuthority {
    pub mint_auth: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintMintFreezeAuthority {
    pub mint_freeze_auth: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintMintDecimals {
    pub decimals: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintTokenBump {
    pub bump: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintProgramSeed {
    pub program_seed: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintAssociatedToken {
    pub wallet: Expr,
    pub mint: Expr,
    pub token_program: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintTokenAccountGroup {
    pub mint: Option<Expr>,
    pub authority: Option<Expr>,
    pub token_program: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstraintTokenMintGroup {
    pub decimals: Option<Expr>,
    pub mint_authority: Option<Expr>,
    pub freeze_authority: Option<Expr>,
    pub token_program: Option<Expr>,
}

// Syntaxt context object for preserving metadata about the inner item.
#[derive(Debug, Clone)]
pub struct Context<T> {
    span: Span,
    inner: T,
}

impl<T> Context<T> {
    pub fn new(span: Span, inner: T) -> Self {
        Self { span, inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Deref for Context<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Spanned for Context<T> {
    fn span(&self) -> Span {
        self.span
    }
}
