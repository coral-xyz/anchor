#[cfg(feature = "idl")]
use crate::idl::{IdlAccount, IdlAccountItem, IdlAccounts};
use anyhow::Result;
use codegen::accounts as accounts_codegen;
use codegen::program as program_codegen;
#[cfg(feature = "idl")]
use heck::MixedCase;
use parser::accounts as accounts_parser;
use parser::program as program_parser;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use std::ops::Deref;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, Ident, ItemMod, ItemStruct, LitStr, Token};

pub mod codegen;
#[cfg(feature = "hash")]
pub mod hash;
#[cfg(not(feature = "hash"))]
pub(crate) mod hash;
#[cfg(feature = "idl")]
pub mod idl;
pub mod parser;

#[derive(Debug)]
pub struct Program {
    pub state: Option<State>,
    pub ixs: Vec<Ix>,
    pub name: syn::Ident,
    pub program_mod: syn::ItemMod,
}

impl Parse for Program {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let program_mod = <ItemMod as Parse>::parse(input)?;
        program_parser::parse(program_mod)
    }
}

impl From<&Program> for proc_macro2::TokenStream {
    fn from(program: &Program) -> Self {
        program_codegen::generate(program)
    }
}

impl ToTokens for Program {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend::<proc_macro2::TokenStream>(self.into());
    }
}

// State struct singleton.
#[derive(Debug)]
pub struct State {
    pub name: String,
    pub strct: syn::ItemStruct,
    pub ctor_and_anchor: Option<(syn::ImplItemMethod, syn::Ident)>,
    pub impl_block_and_methods: Option<(syn::ItemImpl, Vec<StateIx>)>,
    pub interfaces: Option<Vec<StateInterface>>,
    pub is_zero_copy: bool,
}

#[derive(Debug)]
pub struct StateIx {
    pub raw_method: syn::ImplItemMethod,
    pub ident: syn::Ident,
    pub args: Vec<IxArg>,
    pub anchor_ident: syn::Ident,
    // True if there exists a &self on the method.
    pub has_receiver: bool,
}

#[derive(Debug)]
pub struct StateInterface {
    pub trait_name: String,
    pub methods: Vec<StateIx>,
}

#[derive(Debug)]
pub struct Ix {
    pub raw_method: syn::ItemFn,
    pub ident: syn::Ident,
    pub args: Vec<IxArg>,
    // The ident for the struct deriving Accounts.
    pub anchor_ident: syn::Ident,
}

#[derive(Debug)]
pub struct IxArg {
    pub name: proc_macro2::Ident,
    pub raw_arg: syn::PatType,
}

#[derive(Debug)]
pub struct AccountsStruct {
    // Name of the accounts struct.
    pub ident: syn::Ident,
    // Generics + lifetimes on the accounts struct.
    pub generics: syn::Generics,
    // Fields on the accounts struct.
    pub fields: Vec<AccountField>,
}

impl Parse for AccountsStruct {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let strct = <ItemStruct as Parse>::parse(input)?;
        accounts_parser::parse(&strct)
    }
}

impl From<&AccountsStruct> for proc_macro2::TokenStream {
    fn from(accounts: &AccountsStruct) -> Self {
        accounts_codegen::generate(accounts)
    }
}

impl ToTokens for AccountsStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend::<proc_macro2::TokenStream>(self.into());
    }
}

impl AccountsStruct {
    pub fn new(strct: syn::ItemStruct, fields: Vec<AccountField>) -> Self {
        let ident = strct.ident.clone();
        let generics = strct.generics;
        Self {
            ident,
            generics,
            fields,
        }
    }

    // Returns all program owned accounts in the Accounts struct.
    //
    // `global_accs` is given to "link" account types that are embedded
    // in each other.
    pub fn account_tys(
        &self,
        global_accs: &HashMap<String, AccountsStruct>,
    ) -> Result<Vec<String>> {
        let mut tys = vec![];
        for f in &self.fields {
            match f {
                AccountField::Field(f) => {
                    if let Ty::ProgramAccount(pty) = &f.ty {
                        tys.push(pty.account_ident.to_string());
                    }
                }
                AccountField::CompositeField(comp_f) => {
                    let accs = global_accs.get(&comp_f.symbol).ok_or_else(|| {
                        anyhow::format_err!("Invalid account type: {}", comp_f.symbol)
                    })?;
                    tys.extend(accs.account_tys(global_accs)?);
                }
            }
        }
        Ok(tys)
    }

    #[cfg(feature = "idl")]
    pub fn idl_accounts(
        &self,
        global_accs: &HashMap<String, AccountsStruct>,
    ) -> Vec<IdlAccountItem> {
        self.fields
            .iter()
            .map(|acc: &AccountField| match acc {
                AccountField::CompositeField(comp_f) => {
                    let accs_strct = global_accs
                        .get(&comp_f.symbol)
                        .expect("Could not resolve Accounts symbol");
                    let accounts = accs_strct.idl_accounts(global_accs);
                    IdlAccountItem::IdlAccounts(IdlAccounts {
                        name: comp_f.ident.to_string().to_mixed_case(),
                        accounts,
                    })
                }
                AccountField::Field(acc) => IdlAccountItem::IdlAccount(IdlAccount {
                    name: acc.ident.to_string().to_mixed_case(),
                    is_mut: acc.constraints.is_mutable(),
                    is_signer: acc.constraints.is_signer(),
                }),
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub enum AccountField {
    Field(Field),
    CompositeField(CompositeField),
}

#[derive(Debug)]
pub struct Field {
    pub ident: syn::Ident,
    pub ty: Ty,
    pub constraints: ConstraintGroup,
}

#[derive(Debug)]
pub struct CompositeField {
    pub ident: syn::Ident,
    pub symbol: String,
    pub constraints: ConstraintGroup,
    pub raw_field: syn::Field,
}

impl Field {
    pub fn typed_ident(&self) -> proc_macro2::TokenStream {
        let name = &self.ident;

        let ty = match &self.ty {
            Ty::AccountInfo => quote! { AccountInfo },
            Ty::ProgramState(ty) => {
                let account = &ty.account_ident;
                quote! {
                    ProgramState<#account>
                }
            }
            Ty::CpiState(ty) => {
                let account = &ty.account_ident;
                quote! {
                    CpiState<#account>
                }
            }
            Ty::ProgramAccount(ty) => {
                let account = &ty.account_ident;
                quote! {
                    ProgramAccount<#account>
                }
            }
            Ty::Loader(ty) => {
                let account = &ty.account_ident;
                quote! {
                    Loader<#account>
                }
            }
            Ty::CpiAccount(ty) => {
                let account = &ty.account_ident;
                quote! {
                    CpiAccount<#account>
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
        };

        quote! {
            #name: #ty
        }
    }
}

// A type of an account field.
#[derive(Debug, PartialEq)]
pub enum Ty {
    AccountInfo,
    ProgramState(ProgramStateTy),
    CpiState(CpiStateTy),
    ProgramAccount(ProgramAccountTy),
    Loader(LoaderTy),
    CpiAccount(CpiAccountTy),
    Sysvar(SysvarTy),
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct ProgramStateTy {
    pub account_ident: syn::Ident,
}

#[derive(Debug, PartialEq)]
pub struct CpiStateTy {
    pub account_ident: syn::Ident,
}

#[derive(Debug, PartialEq)]
pub struct ProgramAccountTy {
    // The struct type of the account.
    pub account_ident: syn::Ident,
}

#[derive(Debug, PartialEq)]
pub struct CpiAccountTy {
    // The struct type of the account.
    pub account_ident: syn::Ident,
}

#[derive(Debug, PartialEq)]
pub struct LoaderTy {
    // The struct type of the account.
    pub account_ident: syn::Ident,
}

#[derive(Debug, Default, Clone)]
pub struct ConstraintGroup {
    init: Option<Context<ConstraintInit>>,
    mutable: Option<Context<ConstraintMut>>,
    signer: Option<Context<ConstraintSigner>>,
    belongs_to: Vec<Context<ConstraintBelongsTo>>,
    literal: Vec<Context<ConstraintLiteral>>,
    raw: Vec<Context<ConstraintRaw>>,
    owner: Option<Context<ConstraintOwner>>,
    rent_exempt: Option<Context<ConstraintRentExempt>>,
    seeds: Option<Context<ConstraintSeeds>>,
    executable: Option<Context<ConstraintExecutable>>,
    state: Option<Context<ConstraintState>>,
    associated: Option<ConstraintAssociatedGroup>,
}

impl ConstraintGroup {
    pub fn is_init(&self) -> bool {
        self.init.is_some()
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable.is_some()
    }

    pub fn is_signer(&self) -> bool {
        self.signer.is_some()
    }

    pub fn to_vec(self) -> Vec<Constraint> {
        let ConstraintGroup {
            init,
            mutable,
            signer,
            belongs_to,
            literal,
            raw,
            owner,
            rent_exempt,
            seeds,
            executable,
            state,
            associated,
        } = self;

        let mut constraints = Vec::new();

        // The associated cosntraint should always be first since it creates
        // the account (if also init).
        if let Some(c) = associated {
            constraints.push(Constraint::AssociatedGroup(c));
        }
        if let Some(c) = init {
            constraints.push(Constraint::Init(c));
        }
        if let Some(c) = mutable {
            constraints.push(Constraint::Mut(c));
        }
        if let Some(c) = signer {
            constraints.push(Constraint::Signer(c));
        }
        constraints.append(
            &mut belongs_to
                .into_iter()
                .map(|c| Constraint::BelongsTo(c))
                .collect(),
        );
        constraints.append(
            &mut literal
                .into_iter()
                .map(|c| Constraint::Literal(c))
                .collect(),
        );
        constraints.append(&mut raw.into_iter().map(|c| Constraint::Raw(c)).collect());
        if let Some(c) = owner {
            constraints.push(Constraint::Owner(c));
        }
        if let Some(c) = rent_exempt {
            constraints.push(Constraint::RentExempt(c));
        }
        if let Some(c) = seeds {
            constraints.push(Constraint::Seeds(c));
        }
        if let Some(c) = executable {
            constraints.push(Constraint::Executable(c));
        }
        if let Some(c) = state {
            constraints.push(Constraint::State(c));
        }
        constraints
    }
}

#[derive(Debug)]
pub enum Constraint {
    Init(Context<ConstraintInit>),
    Mut(Context<ConstraintMut>),
    Signer(Context<ConstraintSigner>),
    BelongsTo(Context<ConstraintBelongsTo>),
    Literal(Context<ConstraintLiteral>),
    Raw(Context<ConstraintRaw>),
    Owner(Context<ConstraintOwner>),
    RentExempt(Context<ConstraintRentExempt>),
    Seeds(Context<ConstraintSeeds>),
    Executable(Context<ConstraintExecutable>),
    State(Context<ConstraintState>),
    AssociatedGroup(ConstraintAssociatedGroup),
    Associated(Context<ConstraintAssociated>),
    AssociatedPayer(Context<ConstraintAssociatedPayer>),
    AssociatedSpace(Context<ConstraintAssociatedSpace>),
    AssociatedWith(Context<ConstraintAssociatedWith>),
}

impl Parse for Constraint {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        accounts_parser::constraint::parse(stream)
    }
}

#[derive(Debug, Clone)]
pub struct ConstraintInit {}

#[derive(Debug, Clone)]
pub struct ConstraintMut {}

#[derive(Debug, Clone)]
pub struct ConstraintSigner {}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConstraintBelongsTo {
    pub join_target: proc_macro2::Ident,
}

#[derive(Debug, Clone)]
pub struct ConstraintLiteral {
    pub lit: LitStr,
}

#[derive(Debug, Clone)]
pub struct ConstraintRaw {
    pub raw: Expr,
}

#[derive(Debug, Clone)]
pub struct ConstraintOwner {
    pub owner_target: proc_macro2::Ident,
}

#[derive(Debug, Clone)]
pub enum ConstraintRentExempt {
    Enforce,
    Skip,
}

#[derive(Debug, Clone)]
pub struct ConstraintSeeds {
    pub seeds: Punctuated<Expr, Token![,]>,
}

#[derive(Debug, Clone)]
pub struct ConstraintExecutable {}

#[derive(Debug, Clone)]
pub struct ConstraintState {
    pub program_target: proc_macro2::Ident,
}

#[derive(Debug, Clone)]
pub struct ConstraintAssociatedGroup {
    pub is_init: bool,
    pub associated_target: proc_macro2::Ident,
    pub associated_seeds: Vec<syn::Ident>,
    pub payer: Option<syn::Ident>,
    pub space: Option<syn::LitInt>,
}

#[derive(Debug)]
pub struct ConstraintAssociated {
    pub target: proc_macro2::Ident,
}

#[derive(Debug)]
pub struct ConstraintAssociatedPayer {
    pub target: proc_macro2::Ident,
}

#[derive(Debug)]
pub struct ConstraintAssociatedWith {
    pub target: proc_macro2::Ident,
}

#[derive(Debug)]
pub struct ConstraintAssociatedSpace {
    pub space: syn::LitInt,
}

#[derive(Debug)]
pub struct Error {
    pub name: String,
    pub raw_enum: syn::ItemEnum,
    pub ident: syn::Ident,
    pub codes: Vec<ErrorCode>,
}

#[derive(Debug)]
pub struct ErrorCode {
    pub id: u32,
    pub ident: Ident,
    pub msg: Option<String>,
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
}

impl<T> Deref for Context<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Spanned for Context<T> {
    fn span(&self) -> proc_macro2::Span {
        self.span
    }
}
