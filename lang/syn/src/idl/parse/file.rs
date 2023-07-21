use crate::idl::types::*;
use crate::parser::context::CrateContext;
use crate::parser::{self, accounts, docs, error, program};
use crate::Ty;
use crate::{AccountField, AccountsStruct};
use anyhow::Result;
use heck::MixedCase;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use syn::{
    Expr, ExprLit, ItemConst,
    Lit::{Byte, ByteStr},
};

use super::relations;

const DERIVE_NAME: &str = "Accounts";
// TODO: share this with `anchor_lang` crate.
const ERROR_CODE_OFFSET: u32 = 6000;

// Parse an entire interface file.
pub fn parse(
    filename: impl AsRef<Path>,
    version: String,
    seeds_feature: bool,
    no_docs: bool,
    safety_checks: bool,
) -> Result<Option<Idl>> {
    let ctx = CrateContext::parse(filename)?;
    if safety_checks {
        ctx.safety_checks()?;
    }

    let program_mod = match parse_program_mod(&ctx) {
        None => return Ok(None),
        Some(m) => m,
    };
    let mut p = program::parse(program_mod)?;

    if no_docs {
        p.docs = None;
        for ix in &mut p.ixs {
            ix.docs = None;
        }
    }

    let accs = parse_account_derives(&ctx);

    let error = parse_error_enum(&ctx).map(|mut e| error::parse(&mut e, None));
    let error_codes = error.as_ref().map(|e| {
        e.codes
            .iter()
            .map(|code| IdlErrorCode {
                code: ERROR_CODE_OFFSET + code.id,
                name: code.ident.to_string(),
                msg: code.msg.clone(),
            })
            .collect::<Vec<IdlErrorCode>>()
    });

    let instructions = p
        .ixs
        .iter()
        .map(|ix| {
            let args = ix
                .args
                .iter()
                .map(|arg| {
                    let doc = if !no_docs {
                        docs::parse(&arg.raw_arg.attrs)
                    } else {
                        None
                    };
                    IdlField {
                        name: arg.name.to_string().to_mixed_case(),
                        docs: doc,
                        ty: to_idl_type(&ctx, &arg.raw_arg.ty),
                    }
                })
                .collect::<Vec<_>>();
            // todo: don't unwrap
            let accounts_strct = accs.get(&ix.anchor_ident.to_string()).unwrap();
            let accounts = idl_accounts(&ctx, accounts_strct, &accs, seeds_feature, no_docs);
            let ret_type_str = ix.returns.ty.to_token_stream().to_string();
            let returns = match ret_type_str.as_str() {
                "()" => None,
                _ => Some(ret_type_str.parse().unwrap()),
            };
            IdlInstruction {
                name: ix.ident.to_string().to_mixed_case(),
                docs: ix.docs.clone(),
                accounts,
                args,
                returns,
            }
        })
        .collect::<Vec<_>>();

    let events = parse_events(&ctx)
        .iter()
        .map(|e: &&syn::ItemStruct| {
            let fields = match &e.fields {
                syn::Fields::Named(n) => n,
                _ => panic!("Event fields must be named"),
            };
            let fields = fields
                .named
                .iter()
                .map(|f: &syn::Field| {
                    let index = match f.attrs.get(0) {
                        None => false,
                        Some(i) => parser::tts_to_string(&i.path) == "index",
                    };
                    IdlEventField {
                        name: f.ident.clone().unwrap().to_string().to_mixed_case(),
                        ty: to_idl_type(&ctx, &f.ty),
                        index,
                    }
                })
                .collect::<Vec<IdlEventField>>();

            IdlEvent {
                name: e.ident.to_string(),
                fields,
            }
        })
        .collect::<Vec<IdlEvent>>();

    // All user defined types.
    let mut accounts = vec![];
    let mut types = vec![];
    let ty_defs = parse_ty_defs(&ctx, no_docs)?;

    let account_structs = parse_accounts(&ctx);
    let account_names: HashSet<String> = account_structs
        .iter()
        .map(|a| a.ident.to_string())
        .collect::<HashSet<_>>();

    let error_name = error.map(|e| e.name).unwrap_or_else(|| "".to_string());

    // All types that aren't in the accounts section, are in the types section.
    for ty_def in ty_defs {
        // Don't add the error type to the types or accounts sections.
        if ty_def.name != error_name {
            if account_names.contains(&ty_def.name) {
                accounts.push(ty_def);
            } else if !events.iter().any(|e| e.name == ty_def.name) {
                types.push(ty_def);
            }
        }
    }

    let constants = parse_consts(&ctx)
        .iter()
        .map(|c: &&syn::ItemConst| to_idl_const(c))
        .collect::<Vec<IdlConst>>();

    Ok(Some(Idl {
        version,
        name: p.name.to_string(),
        docs: p.docs.clone(),
        instructions,
        types,
        accounts,
        events: if events.is_empty() {
            None
        } else {
            Some(events)
        },
        errors: error_codes,
        metadata: None,
        constants,
    }))
}

// Parse the main program mod.
fn parse_program_mod(ctx: &CrateContext) -> Option<syn::ItemMod> {
    let root = ctx.root_module();
    let mods = root
        .items()
        .filter_map(|i| match i {
            syn::Item::Mod(item_mod) => {
                let mod_count = item_mod
                    .attrs
                    .iter()
                    .filter(|attr| attr.path.segments.last().unwrap().ident == "program")
                    .count();
                if mod_count != 1 {
                    return None;
                }
                Some(item_mod)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if mods.len() != 1 {
        return None;
    }
    Some(mods[0].clone())
}

fn parse_error_enum(ctx: &CrateContext) -> Option<syn::ItemEnum> {
    ctx.enums()
        .find(|item_enum| {
            let attrs_count = item_enum
                .attrs
                .iter()
                .filter(|attr| {
                    let segment = attr.path.segments.last().unwrap();
                    segment.ident == "error_code"
                })
                .count();
            match attrs_count {
                0 => false,
                1 => true,
                _ => panic!("Invalid syntax: one error attribute allowed"),
            }
        })
        .cloned()
}

fn parse_events(ctx: &CrateContext) -> Vec<&syn::ItemStruct> {
    ctx.structs()
        .filter(|item_strct| {
            let attrs_count = item_strct
                .attrs
                .iter()
                .filter(|attr| {
                    let segment = attr.path.segments.last().unwrap();
                    segment.ident == "event"
                })
                .count();
            match attrs_count {
                0 => false,
                1 => true,
                _ => panic!("Invalid syntax: one event attribute allowed"),
            }
        })
        .collect()
}

fn parse_accounts(ctx: &CrateContext) -> Vec<&syn::ItemStruct> {
    ctx.structs()
        .filter(|item_strct| {
            let attrs_count = item_strct
                .attrs
                .iter()
                .filter(|attr| {
                    let segment = attr.path.segments.last().unwrap();
                    segment.ident == "account" || segment.ident == "associated"
                })
                .count();
            match attrs_count {
                0 => false,
                1 => true,
                _ => panic!("Invalid syntax: one account attribute allowed"),
            }
        })
        .collect()
}

// Parse all structs implementing the `Accounts` trait.
fn parse_account_derives(ctx: &CrateContext) -> HashMap<String, AccountsStruct> {
    // TODO: parse manual implementations. Currently we only look
    //       for derives.
    ctx.structs()
        .filter_map(|i_strct| {
            for attr in &i_strct.attrs {
                if attr.path.is_ident("derive") && attr.tokens.to_string().contains(DERIVE_NAME) {
                    let strct = accounts::parse(i_strct).expect("Code not parseable");
                    return Some((strct.ident.to_string(), strct));
                }
            }
            None
        })
        .collect()
}

fn parse_consts(ctx: &CrateContext) -> Vec<&syn::ItemConst> {
    ctx.consts()
        .filter(|item_strct| {
            for attr in &item_strct.attrs {
                if attr.path.segments.last().unwrap().ident == "constant" {
                    return true;
                }
            }
            false
        })
        .collect()
}

// Parse all user defined types in the file.
fn parse_ty_defs(ctx: &CrateContext, no_docs: bool) -> Result<Vec<IdlTypeDefinition>> {
    ctx.structs()
        .filter_map(|item_strct| {
            // Only take serializable types
            let serializable = item_strct.attrs.iter().any(|attr| {
                let attr_string = attr.tokens.to_string();
                let attr_name = attr.path.segments.last().unwrap().ident.to_string();
                let attr_serializable = ["account", "associated", "event", "zero_copy"];

                let derived_serializable = attr_name == "derive"
                    && attr_string.contains("AnchorSerialize")
                    && attr_string.contains("AnchorDeserialize");

                attr_serializable.iter().any(|a| *a == attr_name) || derived_serializable
            });

            if !serializable {
                return None;
            }

            // Only take public types
            match &item_strct.vis {
                syn::Visibility::Public(_) => (),
                _ => return None,
            }

            let name = item_strct.ident.to_string();
            let doc = if !no_docs {
                docs::parse(&item_strct.attrs)
            } else {
                None
            };
            let fields = match &item_strct.fields {
                syn::Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|f: &syn::Field| {
                        let doc = if !no_docs {
                            docs::parse(&f.attrs)
                        } else {
                            None
                        };
                        Ok(IdlField {
                            name: f.ident.as_ref().unwrap().to_string().to_mixed_case(),
                            docs: doc,
                            ty: to_idl_type(ctx, &f.ty),
                        })
                    })
                    .collect::<Result<Vec<IdlField>>>(),
                syn::Fields::Unnamed(_) => return None,
                _ => panic!("Empty structs are allowed."),
            };

            Some(fields.map(|fields| IdlTypeDefinition {
                name,
                generics: None,
                docs: doc,
                ty: IdlTypeDefinitionTy::Struct { fields },
            }))
        })
        .chain(ctx.enums().filter_map(|enm| {
            // Only take public types
            match &enm.vis {
                syn::Visibility::Public(_) => (),
                _ => return None,
            }
            let name = enm.ident.to_string();
            let doc = if !no_docs {
                docs::parse(&enm.attrs)
            } else {
                None
            };
            let variants = enm
                .variants
                .iter()
                .map(|variant: &syn::Variant| {
                    let name = variant.ident.to_string();
                    let fields = match &variant.fields {
                        syn::Fields::Unit => None,
                        syn::Fields::Unnamed(fields) => {
                            let fields: Vec<IdlType> = fields
                                .unnamed
                                .iter()
                                .map(|f| to_idl_type(ctx, &f.ty))
                                .collect();
                            Some(EnumFields::Tuple(fields))
                        }
                        syn::Fields::Named(fields) => {
                            let fields: Vec<IdlField> = fields
                                .named
                                .iter()
                                .map(|f: &syn::Field| {
                                    let name = f.ident.as_ref().unwrap().to_string();
                                    let doc = if !no_docs {
                                        docs::parse(&f.attrs)
                                    } else {
                                        None
                                    };
                                    let ty = to_idl_type(ctx, &f.ty);
                                    IdlField {
                                        name,
                                        docs: doc,
                                        ty,
                                    }
                                })
                                .collect();
                            Some(EnumFields::Named(fields))
                        }
                    };
                    IdlEnumVariant { name, fields }
                })
                .collect::<Vec<IdlEnumVariant>>();
            Some(Ok(IdlTypeDefinition {
                name,
                generics: None,
                docs: doc,
                ty: IdlTypeDefinitionTy::Enum { variants },
            }))
        }))
        .collect()
}

// Replace variable array lengths with values
fn resolve_variable_array_lengths(ctx: &CrateContext, mut tts_string: String) -> String {
    for constant in ctx.consts().filter(|c| match *c.ty {
        // Filter to only those consts that are of type usize or could be cast to usize
        syn::Type::Path(ref p) => {
            let segment = p.path.segments.last().unwrap();
            matches!(
                segment.ident.to_string().as_str(),
                "usize"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "isize"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
            )
        }
        _ => false,
    }) {
        let mut check_string = tts_string.clone();
        // Strip whitespace to handle accidental double whitespaces
        check_string.retain(|c| !c.is_whitespace());
        let size_string = format!("{}]", &constant.ident.to_string());
        let cast_size_string = format!("{}asusize]", &constant.ident.to_string());
        // Check for something to replace
        let mut replacement_string = None;
        if check_string.contains(cast_size_string.as_str()) {
            replacement_string = Some(cast_size_string);
        } else if check_string.contains(size_string.as_str()) {
            replacement_string = Some(size_string);
        }
        if let Some(replacement_string) = replacement_string {
            // Check for the existence of consts existing elsewhere in the
            // crate which have the same name, are usize, and have a
            // different value. We can't know which was intended for the
            // array size from ctx.
            if ctx.consts().any(|c| {
                c != constant
                    && c.ident == constant.ident
                    && c.ty == constant.ty
                    && c.expr != constant.expr
            }) {
                panic!("Crate wide unique name required for array size const.");
            }
            // Replace the match, don't break because there might be multiple replacements to be
            // made in the case of multidimensional arrays
            tts_string = check_string.replace(
                &replacement_string,
                format!("{}]", &constant.expr.to_token_stream()).as_str(),
            );
        }
    }
    tts_string
}

fn to_idl_type(ctx: &CrateContext, ty: &syn::Type) -> IdlType {
    let mut tts_string = parser::tts_to_string(ty);
    if tts_string.starts_with('[') {
        tts_string = resolve_variable_array_lengths(ctx, tts_string);
    }
    // Box<FooType> -> FooType
    tts_string = tts_string
        .strip_prefix("Box < ")
        .and_then(|t| t.strip_suffix(" >"))
        .unwrap_or(&tts_string)
        .into();

    tts_string.parse().unwrap()
}

// TODO parse other issues
fn to_idl_const(item: &ItemConst) -> IdlConst {
    let name = item.ident.to_string();

    if let Expr::Lit(ExprLit { lit, .. }) = &*item.expr {
        match lit {
            ByteStr(lit_byte_str) => {
                return IdlConst {
                    name,
                    ty: IdlType::Bytes,
                    value: format!("{:?}", lit_byte_str.value()),
                }
            }
            Byte(lit_byte) => {
                return IdlConst {
                    name,
                    ty: IdlType::U8,
                    value: lit_byte.value().to_string(),
                }
            }
            _ => (),
        }
    }

    IdlConst {
        name,
        ty: item.ty.to_token_stream().to_string().parse().unwrap(),
        value: item.expr.to_token_stream().to_string().parse().unwrap(),
    }
}

fn idl_accounts(
    ctx: &CrateContext,
    accounts: &AccountsStruct,
    global_accs: &HashMap<String, AccountsStruct>,
    seeds_feature: bool,
    no_docs: bool,
) -> Vec<IdlAccountItem> {
    accounts
        .fields
        .iter()
        .map(|acc: &AccountField| match acc {
            AccountField::CompositeField(comp_f) => {
                let accs_strct = global_accs.get(&comp_f.symbol).unwrap_or_else(|| {
                    panic!("Could not resolve Accounts symbol {}", comp_f.symbol)
                });
                let accounts = idl_accounts(ctx, accs_strct, global_accs, seeds_feature, no_docs);
                IdlAccountItem::IdlAccounts(IdlAccounts {
                    name: comp_f.ident.to_string().to_mixed_case(),
                    accounts,
                })
            }
            AccountField::Field(acc) => IdlAccountItem::IdlAccount(IdlAccount {
                name: acc.ident.to_string().to_mixed_case(),
                is_mut: acc.constraints.is_mutable(),
                is_signer: match acc.ty {
                    Ty::Signer => true,
                    _ => acc.constraints.is_signer(),
                },
                is_optional: if acc.is_optional { Some(true) } else { None },
                docs: if !no_docs { acc.docs.clone() } else { None },
                pda: super::pda::parse(ctx, accounts, acc, seeds_feature),
                relations: relations::parse(acc, seeds_feature),
            }),
        })
        .collect::<Vec<_>>()
}
