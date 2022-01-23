use crate::idl::*;
use crate::parser::context::CrateContext;
use crate::parser::{self, accounts, error, program};
use crate::ConstraintSeedsGroup;
use crate::Ty;
use crate::{AccountField, AccountsStruct, StateIx};
use anyhow::Result;
use heck::MixedCase;
use quote::ToTokens;
use std::str::FromStr;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use syn::Expr;

const DERIVE_NAME: &str = "Accounts";
// TODO: sharee this with `anchor_lang` crate.
const ERROR_CODE_OFFSET: u32 = 6000;

// Parse an entire interface file.
pub fn parse(
    filename: impl AsRef<Path>,
    version: String,
    seeds_feature: bool,
) -> Result<Option<Idl>> {
    let ctx = CrateContext::parse(filename)?;

    let program_mod = match parse_program_mod(&ctx) {
        None => return Ok(None),
        Some(m) => m,
    };
    let p = program::parse(program_mod)?;

    let accs = parse_account_derives(&ctx);

    let state = match p.state {
        None => None,
        Some(state) => match state.ctor_and_anchor {
            None => None, // State struct defined but no implementation
            Some((ctor, anchor_ident)) => {
                let mut methods = state
                    .impl_block_and_methods
                    .map(|(_impl_block, methods)| {
                        methods
                            .iter()
                            .map(|method: &StateIx| {
                                let name = method.ident.to_string().to_mixed_case();
                                let args = method
                                    .args
                                    .iter()
                                    .map(|arg| {
                                        let mut tts = proc_macro2::TokenStream::new();
                                        arg.raw_arg.ty.to_tokens(&mut tts);
                                        let ty = tts.to_string().parse().unwrap();
                                        IdlField {
                                            name: arg.name.to_string().to_mixed_case(),
                                            ty,
                                        }
                                    })
                                    .collect::<Vec<_>>();
                                let accounts_strct =
                                    accs.get(&method.anchor_ident.to_string()).unwrap();
                                let accounts =
                                    idl_accounts(seeds_feature, &ctx, accounts_strct, &accs);
                                IdlInstruction {
                                    name,
                                    accounts,
                                    args,
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let ctor = {
                    let name = "new".to_string();
                    let args = ctor
                        .sig
                        .inputs
                        .iter()
                        .filter(|arg| match arg {
                            syn::FnArg::Typed(pat_ty) => {
                                // TODO: this filtering should be done in the parser.
                                let mut arg_str = parser::tts_to_string(&pat_ty.ty);
                                arg_str.retain(|c| !c.is_whitespace());
                                !arg_str.starts_with("Context<")
                            }
                            _ => false,
                        })
                        .map(|arg: &syn::FnArg| match arg {
                            syn::FnArg::Typed(arg_typed) => {
                                let mut tts = proc_macro2::TokenStream::new();
                                arg_typed.ty.to_tokens(&mut tts);
                                let ty = tts.to_string().parse().unwrap();
                                IdlField {
                                    name: parser::tts_to_string(&arg_typed.pat).to_mixed_case(),
                                    ty,
                                }
                            }
                            _ => panic!("Invalid syntax"),
                        })
                        .collect();
                    let accounts_strct = accs.get(&anchor_ident.to_string()).unwrap();
                    let accounts = idl_accounts(seeds_feature, &ctx, accounts_strct, &accs);
                    IdlInstruction {
                        name,
                        accounts,
                        args,
                    }
                };

                methods.insert(0, ctor);

                let strct = {
                    let fields = match state.strct.fields {
                        syn::Fields::Named(f_named) => f_named
                            .named
                            .iter()
                            .map(|f: &syn::Field| {
                                let mut tts = proc_macro2::TokenStream::new();
                                f.ty.to_tokens(&mut tts);
                                let ty = tts.to_string().parse().unwrap();
                                IdlField {
                                    name: f.ident.as_ref().unwrap().to_string().to_mixed_case(),
                                    ty,
                                }
                            })
                            .collect::<Vec<IdlField>>(),
                        _ => panic!("State must be a struct"),
                    };
                    IdlTypeDefinition {
                        name: state.name,
                        ty: IdlTypeDefinitionTy::Struct { fields },
                    }
                };

                Some(IdlState { strct, methods })
            }
        },
    };
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
                    let mut tts = proc_macro2::TokenStream::new();
                    arg.raw_arg.ty.to_tokens(&mut tts);
                    let ty = tts.to_string().parse().unwrap();
                    IdlField {
                        name: arg.name.to_string().to_mixed_case(),
                        ty,
                    }
                })
                .collect::<Vec<_>>();
            // todo: don't unwrap
            let accounts_strct = accs.get(&ix.anchor_ident.to_string()).unwrap();
            let accounts = idl_accounts(seeds_feature, &ctx, accounts_strct, &accs);
            IdlInstruction {
                name: ix.ident.to_string().to_mixed_case(),
                accounts,
                args,
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
                        ty: parser::tts_to_string(&f.ty).parse().unwrap(),
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
    let ty_defs = parse_ty_defs(&ctx)?;

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
        .map(|c: &&syn::ItemConst| IdlConst {
            name: c.ident.to_string(),
            ty: c.ty.to_token_stream().to_string().parse().unwrap(),
            value: c.expr.to_token_stream().to_string().parse().unwrap(),
        })
        .collect::<Vec<IdlConst>>();

    Ok(Some(Idl {
        version,
        name: p.name.to_string(),
        state,
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
        .filter_map(|item_enum| {
            let attrs_count = item_enum
                .attrs
                .iter()
                .filter(|attr| {
                    let segment = attr.path.segments.last().unwrap();
                    segment.ident == "error"
                })
                .count();
            match attrs_count {
                0 => None,
                1 => Some(item_enum),
                _ => panic!("Invalid syntax: one error attribute allowed"),
            }
        })
        .next()
        .cloned()
}

fn parse_events(ctx: &CrateContext) -> Vec<&syn::ItemStruct> {
    ctx.structs()
        .filter_map(|item_strct| {
            let attrs_count = item_strct
                .attrs
                .iter()
                .filter(|attr| {
                    let segment = attr.path.segments.last().unwrap();
                    segment.ident == "event"
                })
                .count();
            match attrs_count {
                0 => None,
                1 => Some(item_strct),
                _ => panic!("Invalid syntax: one event attribute allowed"),
            }
        })
        .collect()
}

fn parse_accounts(ctx: &CrateContext) -> Vec<&syn::ItemStruct> {
    ctx.structs()
        .filter_map(|item_strct| {
            let attrs_count = item_strct
                .attrs
                .iter()
                .filter(|attr| {
                    let segment = attr.path.segments.last().unwrap();
                    segment.ident == "account" || segment.ident == "associated"
                })
                .count();
            match attrs_count {
                0 => None,
                1 => Some(item_strct),
                _ => panic!("Invalid syntax: one event attribute allowed"),
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
                if attr.tokens.to_string().contains(DERIVE_NAME) {
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
fn parse_ty_defs(ctx: &CrateContext) -> Result<Vec<IdlTypeDefinition>> {
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
            let fields = match &item_strct.fields {
                syn::Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|f: &syn::Field| {
                        let mut tts = proc_macro2::TokenStream::new();
                        f.ty.to_tokens(&mut tts);
                        // Handle array sizes that are constants
                        let mut tts_string = tts.to_string();
                        if tts_string.starts_with('[') {
                            tts_string = resolve_variable_array_length(ctx, tts_string);
                        }
                        Ok(IdlField {
                            name: f.ident.as_ref().unwrap().to_string().to_mixed_case(),
                            ty: tts_string.parse()?,
                        })
                    })
                    .collect::<Result<Vec<IdlField>>>(),
                syn::Fields::Unnamed(_) => return None,
                _ => panic!("Empty structs are allowed."),
            };

            Some(fields.map(|fields| IdlTypeDefinition {
                name,
                ty: IdlTypeDefinitionTy::Struct { fields },
            }))
        })
        .chain(ctx.enums().map(|enm| {
            let name = enm.ident.to_string();
            let variants = enm
                .variants
                .iter()
                .map(|variant: &syn::Variant| {
                    let name = variant.ident.to_string();
                    let fields = match &variant.fields {
                        syn::Fields::Unit => None,
                        syn::Fields::Unnamed(fields) => {
                            let fields: Vec<IdlType> =
                                fields.unnamed.iter().map(to_idl_type).collect();
                            Some(EnumFields::Tuple(fields))
                        }
                        syn::Fields::Named(fields) => {
                            let fields: Vec<IdlField> = fields
                                .named
                                .iter()
                                .map(|f: &syn::Field| {
                                    let name = f.ident.as_ref().unwrap().to_string();
                                    let ty = to_idl_type(f);
                                    IdlField { name, ty }
                                })
                                .collect();
                            Some(EnumFields::Named(fields))
                        }
                    };
                    IdlEnumVariant { name, fields }
                })
                .collect::<Vec<IdlEnumVariant>>();
            Ok(IdlTypeDefinition {
                name,
                ty: IdlTypeDefinitionTy::Enum { variants },
            })
        }))
        .collect()
}

// Replace variable array lengths with values
fn resolve_variable_array_length(ctx: &CrateContext, tts_string: String) -> String {
    for constant in ctx.consts() {
        if constant.ty.to_token_stream().to_string() == "usize"
            && tts_string.contains(&constant.ident.to_string())
        {
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
            return tts_string.replace(
                &constant.ident.to_string(),
                &constant.expr.to_token_stream().to_string(),
            );
        }
    }
    tts_string
}

fn to_idl_type(f: &syn::Field) -> IdlType {
    let mut tts = proc_macro2::TokenStream::new();
    f.ty.to_tokens(&mut tts);
    tts.to_string().parse().unwrap()
}

fn idl_accounts(
    seeds_feature: bool,
    ctx: &CrateContext,
    accounts: &AccountsStruct,
    global_accs: &HashMap<String, AccountsStruct>,
) -> Vec<IdlAccountItem> {
    accounts
        .fields
        .iter()
        .map(|acc: &AccountField| match acc {
            AccountField::CompositeField(comp_f) => {
                let accs_strct = global_accs
                    .get(&comp_f.symbol)
                    .expect("Could not resolve Accounts symbol");
                let accounts = idl_accounts(seeds_feature, ctx, accs_strct, global_accs);
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
                pda: if seeds_feature {
                    acc.constraints
                        .seeds
                        .as_ref()
                        .map(|s| parse_pda(ctx, accounts, s))
                } else {
                    None
                },
            }),
        })
        .collect::<Vec<_>>()
}

// Parses a seeds constraint, extracting the IdlSeed types.
//
// Note: This implementation is fragile as it makes assumptions about the types
//       that can be used here (e.g., no program-defined function calls in
//       seeds).
//
//       This probably doesn't cover all cases. If you see a warning log, you
//       can add a new case here. In the worst case, we miss a seed and
//       the parser will treat the given seeds as empty and so clients will
//       simply fail to automatically populate the PDA accounts.
//
// Seed Assumptions: Seeds must be of the form
//
// - instruction argument.
// - account context field pubkey.
// - account data, where the account is defined in the current program.
//   We make an exception for the SPL token program, since it is so common
//   and sometimes convenient to use fields as a seed (e.g. Auction house
//   program).
//   In the case of nested structs/account data, all nested structs must
//   be defined in the current program as well.
// - byte string literal (e.g. b"MY_SEED").
// - byte string literal constant  (e.g. `pub const MY_SEED: [u8; 2] = *b"hi";`).
//
fn parse_pda(
    ctx: &CrateContext,
    accounts: &AccountsStruct,
    seeds_grp: &ConstraintSeedsGroup,
) -> IdlPda {
    // All the available seed variables (except for constants).
    let ix_args = accounts.instruction_args().unwrap_or_default();
    let const_names: Vec<String> = ctx.consts().map(|c| c.ident.to_string()).collect();
    let account_field_names = accounts.field_names();

    // Final seeds accumulator.
    let mut seeds = Vec::new();

    // Parse each seed.
    for seed in &seeds_grp.seeds {
        match parse_seed(
            ctx,
            accounts,
            &ix_args,
            &const_names,
            &account_field_names,
            seed,
        ) {
            Some(seed) => seeds.push(seed),
            None => {
                return IdlPda {
                    seeds: Vec::new(),
                    program_id: None,
                }
            }
        }
    }

    // Parse the program id.
    let program_id = seeds_grp
        .program_seed
        .as_ref()
        .map(|pid| {
            parse_seed(
                ctx,
                accounts,
                &ix_args,
                &const_names,
                &account_field_names,
                pid,
            )
        })
        .unwrap_or(None);

    // Done.
    IdlPda { seeds, program_id }
}

fn parse_seed(
    ctx: &CrateContext,
    accounts: &AccountsStruct,
    ix_args: &HashMap<String, String>,
    const_names: &[String],
    account_field_names: &[String],
    seed: &Expr,
) -> Option<IdlSeed> {
    match seed {
        Expr::MethodCall(_) => {
            let (var_name, path) = {
                // Convert the seed into the raw string representation.
                let seed_str = parser::tts_to_string(&seed);

                let mut components: Vec<&str> = seed_str.split(" . ").collect();
                if components.len() <= 1 {
                    println!("WARNING: seeds are in an unexpected format: {:?}", seed);
                    return None;
                }

                // The name of the variable (or field).
                let name = components.remove(0).to_string();

                // The path to the seed (only if the `name` type is a struct).
                let mut path = Vec::new();
                while components.is_empty() {
                    let c = components.remove(0);
                    if c.contains("()") {
                        break;
                    }
                    path.push(c.to_string());
                }

                if path.len() == 1 && (path[0] == "key" || path[0] == "key()") {
                    path = Vec::new();
                }

                (name, path)
            };

            // Instruction argument.
            if ix_args.contains_key(&var_name) {
                let idl_ty = IdlType::from_str(ix_args.get(&var_name).unwrap()).ok()?;
                Some(IdlSeed::Arg(IdlSeedArg {
                    ty: idl_ty,
                    path: match path.len() {
                        0 => var_name,
                        _ => format!("{}.{}", var_name, path.join(".")),
                    },
                }))
            }
            // Constant.
            else if const_names.contains(&var_name) {
                // Pull in the constant value directly into the IDL.
                assert!(path.is_empty());
                let const_item = ctx.consts().find(|c| c.ident == var_name).unwrap();
                let idl_ty = IdlType::from_str(&parser::tts_to_string(&const_item.ty)).ok()?;
                let mut idl_ty_value = parser::tts_to_string(&const_item.expr);

                if let IdlType::Array(_ty, _size) = &idl_ty {
                    if idl_ty_value.contains("b\"") {
                        let components: Vec<&str> = idl_ty_value.split('b').collect();
                        assert!(components.len() == 2);
                        let mut str_lit = components[1].to_string();
                        str_lit.retain(|c| c != '"');
                        idl_ty_value = format!("{:?}", str_lit.as_bytes());
                    }
                }

                Some(IdlSeed::Const(IdlSeedConst {
                    ty: idl_ty,
                    value: serde_json::from_str(&idl_ty_value).unwrap(),
                }))
            }
            // Account pubkey or account data.
            else if account_field_names.contains(&var_name) {
                Some(IdlSeed::Account(IdlSeedAccount {
                    path: match path.len() {
                        0 => var_name.clone(),
                        _ => format!("{}.{}", var_name, path.join(".")),
                    },
                    ty: parse_seed_account_field_ty(ctx, accounts, var_name.clone(), &path)?,
                    account: parse_seed_account_ty(ctx, accounts, var_name),
                }))
            }
            // String literal.
            else if path.is_empty() && var_name.contains('"') {
                let mut var_name = var_name;
                // Remove the byte `b` prefix if the string is of the form `b"seed".
                if var_name.starts_with("b\"") {
                    var_name.remove(0);
                }
                let value_string: String = var_name.chars().filter(|c| *c != '"').collect();
                Some(IdlSeed::Const(IdlSeedConst {
                    value: serde_json::Value::String(value_string),
                    ty: IdlType::String,
                }))
            }
            // Unknown.
            else {
                println!("WARNING: unexpected seed category for var: {:?}", var_name);
                None
            }
        }
        Expr::Reference(expr_reference) => parse_seed(
            ctx,
            accounts,
            ix_args,
            const_names,
            account_field_names,
            &expr_reference.expr,
        ),
        Expr::Index(_) => {
            // Slice literal.
            println!("WARNING: auto pda derivation not currently supported for slice literals");
            None
        }
        _ => {
            // Unknown type. Please file an issue.
            println!("WARNING: unexpected seed: {:?}", seed);
            None
        }
    }
}

fn parse_seed_account_ty(
    _ctx: &CrateContext,
    accounts: &AccountsStruct,
    var_name: String,
) -> Option<String> {
    // Get the anchor account field from the derive accounts struct.
    let account_field = accounts
        .fields
        .iter()
        .find(|field| *field.ident() == var_name)
        .unwrap();

    // Get the struct name from the account field.
    account_field.ty_name()
}

fn parse_seed_account_field_ty(
    ctx: &CrateContext,
    accounts: &AccountsStruct,
    var_name: String,
    mut path: &[String],
) -> Option<IdlType> {
    match path.len() {
        0 => Some(IdlType::PublicKey),
        1 => {
            // Get the anchor account field from the derive accounts struct.
            let account_field = accounts
                .fields
                .iter()
                .find(|field| *field.ident() == var_name)
                .unwrap();

            // Get the struct name from the account field.
            let ty_name = account_field.ty_name()?;

            if ty_name == "TokenAccount" {
                assert!(path.len() == 1);
                let token_field = &path[0];
                if token_field == "mint" {
                    return Some(IdlType::PublicKey);
                }
            }
            // Get the rust representation of the field's struct.
            let strct = ctx.structs().find(|s| s.ident == ty_name).unwrap();

            Some(parse_field_path(ctx, strct, &mut path))
        }
        _ => panic!("invariant violation"),
    }
}

fn parse_field_path(ctx: &CrateContext, strct: &syn::ItemStruct, path: &mut &[String]) -> IdlType {
    let field_name = &path[0];
    *path = &path[1..];

    // Get the type name for the field.
    let next_field = strct
        .fields
        .iter()
        .find(|f| &f.ident.clone().unwrap().to_string() == field_name)
        .unwrap();
    let next_field_ty_str = parser::tts_to_string(&next_field.ty);

    // The path is empty so this must be a primitive type.
    if path.is_empty() {
        return next_field_ty_str.parse().unwrap();
    }

    // Get the rust representation of hte field's struct.
    let strct = ctx
        .structs()
        .find(|s| s.ident == next_field_ty_str)
        .unwrap();

    parse_field_path(ctx, strct, path)
}
