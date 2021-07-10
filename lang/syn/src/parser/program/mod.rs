use crate::{program_argument, Program, ProgramArguments};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Error as ParseError, Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parenthesized, token, Ident, LitStr, Token};

mod instructions;
mod state;

pub fn parse(program_mod: syn::ItemMod) -> ParseResult<Program> {
    let state = state::parse(&program_mod)?;
    let (ixs, fallback_fn) = instructions::parse(&program_mod)?;
    let program_attr = program_mod
        .attrs
        .iter()
        .find(|attr| attr.path == syn::parse_str("program").unwrap())
        .unwrap();
    let program_arguments = syn::parse2(program_attr.tokens.clone())?;
    Ok(Program {
        state,
        ixs,
        name: program_mod.ident.clone(),
        program_mod,
        fallback_fn,
        program_arguments,
    })
}

#[derive(Default)]
struct ProgramArgumentsOption {
    no_entrypoint: Option<LitStr>,
    no_idl: Option<LitStr>,
}
impl From<ProgramArgumentsOption> for ProgramArguments {
    fn from(from: ProgramArgumentsOption) -> Self {
        Self {
            no_entrypoint: from
                .no_entrypoint
                .unwrap_or_else(program_argument::default_no_entrypoint),
            no_idl: from.no_idl.unwrap_or_else(program_argument::default_no_idl),
        }
    }
}
impl Parse for ProgramArguments {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let seperated: Punctuated<ProgramArgument, Token![,]> =
                Punctuated::parse_terminated(&content)?;
            let mut optional = ProgramArgumentsOption::default();
            for argument in seperated {
                match argument.clone() {
                    ProgramArgument::NoEntrypoint { name, .. } => {
                        if optional.no_entrypoint.replace(name).is_some() {
                            return Err(ParseError::new_spanned(
                                argument,
                                format!(
                                    "Multiple `{}` arguments, can only have one.",
                                    program_argument::NO_ENTRYPOINT_IDENT
                                ),
                            ));
                        }
                    }
                    ProgramArgument::NoIdl { name, .. } => {
                        if optional.no_idl.replace(name).is_some() {
                            return Err(ParseError::new_spanned(
                                argument,
                                format!(
                                    "Multiple `{}` arguments, can only have one.",
                                    program_argument::NO_IDL_IDENT
                                ),
                            ));
                        }
                    }
                }
            }
            Ok(optional.into())
        } else {
            // Remove this branch if required arguments are added
            Ok(ProgramArgumentsOption::default().into())
        }
    }
}

#[derive(Clone)]
enum ProgramArgument {
    NoEntrypoint {
        ident: Ident,
        equals: Token![=],
        name: LitStr,
    },
    NoIdl {
        ident: Ident,
        equals: Token![=],
        name: LitStr,
    },
}
impl Parse for ProgramArgument {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let ident: Ident = input.parse()?;
        let ident_name = ident.to_string();
        Ok(match ident_name.as_str() {
            program_argument::NO_ENTRYPOINT_IDENT => Self::NoEntrypoint {
                ident,
                equals: input.parse()?,
                name: input.parse()?,
            },
            program_argument::NO_IDL_IDENT => Self::NoIdl {
                ident,
                equals: input.parse()?,
                name: input.parse()?,
            },
            _ => {
                return Err(ParseError::new_spanned(
                    ident,
                    format!("Unknown program argument: {}", ident_name),
                ))
            }
        })
    }
}
impl ToTokens for ProgramArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::NoEntrypoint {
                ident,
                equals,
                name,
            } => {
                ident.to_tokens(tokens);
                equals.to_tokens(tokens);
                name.to_tokens(tokens);
            }
            Self::NoIdl {
                ident,
                equals,
                name,
            } => {
                ident.to_tokens(tokens);
                equals.to_tokens(tokens);
                name.to_tokens(tokens);
            }
        }
    }
}

fn ctx_accounts_ident(path_ty: &syn::PatType) -> ParseResult<proc_macro2::Ident> {
    let p = match &*path_ty.ty {
        syn::Type::Path(p) => &p.path,
        _ => return Err(ParseError::new(path_ty.ty.span(), "invalid type")),
    };
    let segment = p
        .segments
        .first()
        .ok_or_else(|| ParseError::new(p.segments.span(), "expected generic arguments here"))?;

    let generic_args = match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args,
        _ => return Err(ParseError::new(path_ty.span(), "missing accounts context")),
    };
    let generic_ty = generic_args
        .args
        .iter()
        .filter_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .next()
        .ok_or_else(|| ParseError::new(generic_args.span(), "expected Accounts type"))?;

    let path = match generic_ty {
        syn::Type::Path(ty_path) => &ty_path.path,
        _ => {
            return Err(ParseError::new(
                generic_ty.span(),
                "expected Accounts struct type",
            ))
        }
    };
    Ok(path.segments[0].ident.clone())
}
