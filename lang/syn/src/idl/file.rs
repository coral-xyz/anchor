use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::Result;
use cargo::core::Workspace;
use cargo_metadata::{CargoOpt, MetadataCommand};
use heck::{MixedCase, SnakeCase};
use quote::ToTokens;
use syn::{
    Expr, ExprLit, GenericArgument, ItemConst,
    Lit::{Byte, ByteStr},
    PathArguments, Type,
};

use crate::{
    idl::*,
    parser::{self, accounts, context::CrateContext, docs, program},
    AccountField, AccountsStruct, Error, ErrorCode, Ty,
};

// TODO: share this with `anchor_lang` crate.
const ERROR_CODE_OFFSET: u32 = 6000;

// Parse an entire interface file.
pub fn parse(
    crate_root: PathBuf,
    cargo_path: PathBuf,
    version: String,
    seeds_feature: bool,
    no_docs: bool,
    safety_checks: bool,
) -> Result<Option<Idl>> {
    let ctx = CrateContext::parse(crate_root, &None)?;
    if safety_checks {
        ctx.safety_checks()?;
    }

    let program_mod = match parse_program_mod(&ctx)? {
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

    let instructions = p
        .ixs
        .iter()
        .map(|ix| {
            let args: Result<Vec<IdlField>> = ix
                .args
                .iter()
                .filter_map(|arg| {
                    let doc = if !no_docs {
                        docs::parse(&arg.raw_arg.attrs)
                    } else {
                        None
                    };
                    to_idl_type(&ctx, &arg.raw_arg.ty).transpose().map(|ty| {
                        ty.map(|ty| IdlField {
                            name: arg.name.to_string().to_mixed_case(),
                            docs: doc,
                            ty,
                        })
                    })
                })
                .collect();

            let accounts_strct = accs.get(&ix.anchor_ident.to_string()).unwrap();
            let accounts = idl_accounts(&ctx, accounts_strct, &accs, seeds_feature, no_docs);
            let ret_type_str = ix.returns.ty.to_token_stream().to_string();
            let returns = match ret_type_str.as_str() {
                "()" => None,
                _ => IdlType::from_str(&ret_type_str)?,
            };

            Ok(IdlInstruction {
                name: ix.ident.to_string().to_mixed_case(),
                docs: ix.docs.clone(),
                accounts,
                args: args?,
                returns,
            })
        })
        .collect::<Result<Vec<IdlInstruction>>>()?;

    let events = parse_events(&ctx)
        .iter()
        .map(|e: &&syn::ItemStruct| {
            let fields = match &e.fields {
                syn::Fields::Named(n) => n,
                _ => panic!("Event fields must be named"),
            };
            let fields: Result<Vec<IdlEventField>, _> = fields
                .named
                .iter()
                .filter_map(|f: &syn::Field| {
                    let index = match f.attrs.get(0) {
                        None => false,
                        Some(i) => parser::tts_to_string(&i.path) == "index",
                    };
                    to_idl_type(&ctx, &f.ty).transpose().map(|ty| {
                        ty.map(|t| IdlEventField {
                            name: f.ident.clone().unwrap().to_string().to_mixed_case(),
                            ty: t,
                            index,
                        })
                    })
                })
                .collect();

            Ok(IdlEvent {
                name: e.ident.to_string(),
                fields: fields?,
            })
        })
        .collect::<Result<Vec<IdlEvent>>>()?;

    let (error_name, account_serialize_impls, borsh_serialize_impls) = parse_impls(&ctx);
    let error = error_name.map(|name| {
        let (mut error_enum, error_msg_impl) = parse_error(&ctx, name);
        parse_idl_error(&mut error_enum, &error_msg_impl)
    });
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

    // All user defined types.
    let mut accounts = vec![];
    let mut types = vec![];
    let ty_defs = parse_ty_defs(
        &ctx,
        &account_serialize_impls,
        &borsh_serialize_impls,
        no_docs,
    )?;

    let account_structs = parse_accounts(&ctx, &account_serialize_impls);
    let account_names: HashSet<String> = account_structs
        .iter()
        .map(|a| a.ident.to_string())
        .collect::<HashSet<_>>();

    let error_name = error.map(|e| e.name).unwrap_or_else(|| "".to_string());

    // All external types which are imported in our main crate. They might be
    // potentially used as type fields.
    let external_types = parse_external_types(&ctx);

    let mut external_types_to_parse: HashMap<String, String> = HashMap::new();

    // All types that aren't in the accounts section, are in the types section.
    for ty_def in ty_defs {
        // Don't add the error type to the types or accounts sections.
        if ty_def.name != error_name {
            // If type of any field is an external type, we need to parse the
            // crate it comes from and include that type in IDL.
            // TODO(vadorovsky): Handle external types in data-carrying enums.
            if let IdlTypeDefinitionTy::Struct { fields } = &ty_def.ty {
                for field in fields {
                    if let IdlType::Defined(ty) = &field.ty {
                        if let Some(crate_name) = external_types.get(ty) {
                            external_types_to_parse.insert(ty.clone(), crate_name.clone());
                        }
                    }
                }
            }

            if let Some(crate_name) = external_types.get(&ty_def.name) {
                external_types_to_parse.insert(ty_def.name.clone(), crate_name.clone());
            }
            if account_names.contains(&ty_def.name) {
                accounts.push(ty_def);
            } else if !events.iter().any(|e| e.name == ty_def.name) {
                types.push(ty_def);
            }
        }
    }

    if !external_types_to_parse.is_empty() {
        let dependencies = dependencies(&cargo_path)?;

        for (ty_name, crate_name) in external_types_to_parse {
            let dependency = if let Some(dependency) = dependencies.get(&crate_name) {
                dependency
            } else if let Some(dependency) = dependencies.get(&crate_name.replace('_', "-")) {
                dependency
            } else {
                return Err(anyhow::anyhow!(
                    "Crate {} (or {}) not found in Cargo.toml",
                    crate_name.replace('_', "-"),
                    crate_name
                ));
            };

            let mut parsed_types = HashSet::new();
            let crate_ctx = CrateContext::parse(&dependency.path, &dependency.features)?;
            let (_, account_serialize_impls, borsh_serialize_impls) = parse_impls(&crate_ctx);
            let ty_defs = parse_ty_defs(
                &crate_ctx,
                &account_serialize_impls,
                &borsh_serialize_impls,
                no_docs,
            )?;
            for ty_def in ty_defs {
                parsed_types.insert(ty_def.name.clone());
                types.push(ty_def);
            }
            if !parsed_types.contains(&ty_name) {
                return Err(anyhow::anyhow!(
                    "Type {} not found in crate {}",
                    ty_name,
                    crate_name
                ));
            }
        }
    }

    let constants = parse_consts(&ctx)
        .iter()
        .filter_map(|c: &&syn::ItemConst| to_idl_const(c).transpose())
        .collect::<Result<Vec<IdlConst>, _>>()?;

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

/// Dependency of a Rust crate (an another crate).
struct CargoDependency {
    path: PathBuf,
    features: Option<Vec<String>>,
}

/// Parse the Cargo metadata and return a map of dependencies, their source
/// paths (inside Cargo cache), and their enabled features from Cargo metadata
/// of the main crate.
fn dependencies<P>(cargo_path: P) -> Result<HashMap<String, CargoDependency>>
where
    P: AsRef<Path>,
{
    let config = cargo::Config::default()?;
    let ws = Workspace::new(cargo_path.as_ref(), &config)?;

    let dependency_features = ws
        .members()
        .flat_map(|member| {
            member.dependencies().iter().map(move |dependency| {
                let features: Vec<String> = dependency
                    .features()
                    .iter()
                    .map(|feature| feature.to_string())
                    .collect();
                (dependency.package_name().to_string(), features)
            })
        })
        .collect::<HashMap<String, Vec<String>>>();

    // TODO(vadorovsky): We are using `cargo_metadata` crate to parse information
    // about features of dependencies, but it's calling the cargo binary. We
    // should try to avoid that and find the way to use the API of `cargo` crate
    // to retrieve that information just with Rust code.
    let metadata = MetadataCommand::new()
        .manifest_path(cargo_path.as_ref())
        .features(CargoOpt::AllFeatures)
        .exec()?;
    Ok(metadata
        .packages
        .iter()
        .filter(|package| package.id != metadata.workspace_members[0]) // Filter out the current crate
        .filter_map(|package| {
            package.manifest_path.parent().map(|cache_dir| {
                (
                    package.name.clone(),
                    CargoDependency {
                        path: cache_dir.to_path_buf().into_std_path_buf(),
                        features: dependency_features.get(&package.name).cloned(),
                    },
                )
            })
        })
        .collect())
}

/// Parse the main program mod.
fn parse_program_mod(ctx: &CrateContext) -> Result<Option<syn::ItemMod>> {
    let root = ctx.root_module();
    let prog_mods = root
        .items()
        .filter_map(|i| match i {
            syn::Item::Mod(item_mod) => {
                if item_mod.ident == "program" {
                    return Some(item_mod);
                }
                None
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    if prog_mods.len() != 1 {
        return Ok(None);
    }

    let strcts = match prog_mods[0].content {
        Some((_, ref items)) => items
            .iter()
            .filter_map(|i| match i {
                syn::Item::Struct(strct) => Some(strct),
                _ => None,
            })
            .collect::<Vec<_>>(),
        None => return Ok(None),
    };

    if strcts.len() != 1 {
        return Ok(None);
    }

    let mod_ident = strcts[0].ident.to_string().to_snake_case();

    let mods = root
        .items()
        .filter_map(|i| match i {
            syn::Item::Mod(item_mod) => {
                if item_mod.ident != mod_ident {
                    return None;
                }
                Some(item_mod)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    if mods.len() != 1 {
        return Err(anyhow::anyhow!(
            "Expected to find a program mod named `{}`. Please note that program mod name has \
             to be easily convertible between camel case and snake case, therefore names like \
             `name_0` are problematic (it's better to use `name0`).",
            mod_ident
        ));
    }
    Ok(Some(mods[0].clone()))
}

/// Parse the error enum.
fn parse_error(ctx: &CrateContext, error_name: String) -> (syn::ItemEnum, syn::ItemImpl) {
    let mut item_error = None;
    for item_enum in ctx.enums() {
        if item_enum.ident == error_name {
            if item_error.is_some() {
                panic!("Invalid syntax: one error attribute allowed");
            }
            item_error = Some(item_enum.clone());
        }
    }
    let mut item_error_msg_impl = None;
    for item_impl in ctx.impls() {
        // impl std::fmt::Display for ErrorCode
        if let Some((_, path, _)) = &item_impl.trait_ {
            let mut segments = path.segments.iter();
            if segments.next().map_or(false, |s| s.ident == "std")
                && segments.next().map_or(false, |s| s.ident == "fmt")
                && segments.next().map_or(false, |s| s.ident == "Display")
            {
                if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                    if let Some(ident) = type_path.path.get_ident() {
                        if ident == &error_name {
                            item_error_msg_impl = Some(item_impl.clone());
                        }
                    }
                }
            }
        }
    }
    let item_error = item_error.unwrap_or_else(|| {
        // Impossible to happen unless we have a bug in parsing
        // `impl From<...> for anchor_lang::error::Error` blocks.
        panic!("Error enum `{}` not found", error_name)
    });
    let item_error_msg_impl = item_error_msg_impl.unwrap_or_else(|| {
        panic!(
            "`impl std::fmt::Display` block with error messages for `{}` not found",
            item_error.ident
        )
    });
    (item_error, item_error_msg_impl)
}

/// Parse the Rust error enum (to retrieve error names and codes) and
/// `impl std::fmt::Display` block for that enum (to retrieve error messages),
/// convert them into IDL error.
pub fn parse_idl_error(error_enum: &mut syn::ItemEnum, error_msg_impl: &syn::ItemImpl) -> Error {
    let ident = error_enum.ident.clone();
    let mut last_discriminant = 0;

    let error_msgs: HashMap<String, String> = error_msg_impl
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Method(method) = item {
                if method.sig.ident == "fmt" {
                    return method.block.stmts.first();
                }
            }
            None
        })
        .filter_map(|stmt| {
            if let syn::Stmt::Expr(syn::Expr::Match(expr_match)) = stmt {
                return Some(expr_match);
            }
            None
        })
        .flat_map(|expr_match| expr_match.arms.iter())
        .filter_map(|arm| {
            if let syn::Pat::Path(path) = &arm.pat {
                if let Some(last_segment) = path.path.segments.iter().last() {
                    if let syn::Expr::MethodCall(syn::ExprMethodCall { args, .. }) =
                        arm.body.as_ref()
                    {
                        if let Some(syn::Expr::Macro(expr_macro)) = args.iter().next() {
                            if let Some(proc_macro2::TokenTree::Literal(literal)) =
                                expr_macro.mac.tokens.clone().into_iter().next()
                            {
                                return Some((
                                    last_segment.ident.to_string(),
                                    literal.to_string().trim_matches('\"').to_owned(),
                                ));
                            }
                        }
                    }
                }
            }

            None
        })
        .collect();

    let codes: Vec<ErrorCode> = error_enum
        .variants
        .iter_mut()
        .map(|variant| {
            let ident = variant.ident.clone();
            let msg = error_msgs.get(&ident.to_string()).map(|m| m.to_owned());
            let id = match &variant.discriminant {
                None => last_discriminant,
                Some((_, disc)) => match disc {
                    syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                        syn::Lit::Int(int) => {
                            int.base10_parse::<u32>().expect("Must be a base 10 number")
                        }
                        _ => panic!("Invalid error discriminant"),
                    },
                    _ => panic!("Invalid error discriminant"),
                },
            };
            last_discriminant = id + 1;

            // Remove any non-doc attributes on the error variant.
            variant
                .attrs
                .retain(|attr| attr.path.segments[0].ident == "doc");

            ErrorCode { id, ident, msg }
        })
        .collect();
    Error {
        name: error_enum.ident.to_string(),
        raw_enum: error_enum.clone(),
        ident,
        codes,
        args: None,
    }
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

/// Parse `impl` blocks to retrieve:
/// * The name of the error enum (if exists).
/// * Names of types implementing `AnchorSerialize` and/or `AnchorDeserialize` -
///   accounts types.
/// * Names of types implementing `BorshSerialize` and/or `BorshDeserialize` -
///   types which might be potentially used in account fields.
fn parse_impls(ctx: &CrateContext) -> (Option<String>, HashSet<String>, HashSet<String>) {
    let mut error_name = None;
    let mut account_serialize_impls = HashSet::new();
    let mut borsh_serialize_impls = HashSet::new();

    for i_impl in ctx.impls() {
        if let Type::Path(path) = i_impl.self_ty.as_ref() {
            let mut segments = path.path.segments.iter();
            if segments.next().map_or(false, |s| s.ident == "anchor_lang")
                && segments.next().map_or(false, |s| s.ident == "error")
                && segments.next().map_or(false, |s| s.ident == "Error")
            {
                if let Some((_, path, _)) = &i_impl.trait_ {
                    let segments = &mut path.segments.iter();
                    if let Some(segment) = segments.next() {
                        if segment.ident == "From" {
                            if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                                if let Some(GenericArgument::Type(Type::Path(path))) =
                                    arguments.args.iter().next()
                                {
                                    if let Some(segment) = path.path.segments.iter().next() {
                                        error_name = Some(segment.ident.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((_, path, _)) = &i_impl.trait_ {
            let segments = &path.segments;
            if segments.iter().any(|segment| {
                matches!(
                    segment.ident.to_string().as_str(),
                    "AccountSerialize" | "AccountDeserialize"
                )
            }) {
                let r#type = i_impl.self_ty.as_ref();
                if let Type::Path(path) = r#type {
                    if let Some(segment) = path.path.segments.iter().next() {
                        let ident = segment.ident.to_string();
                        if ident != "IdlAccount" {
                            account_serialize_impls.insert(ident);
                        }
                    }
                }
            }
            if segments.iter().any(|segment| {
                matches!(
                    segment.ident.to_string().as_str(),
                    "BorshSerialize" | "BorshDeserialize"
                )
            }) {
                let r#type = i_impl.self_ty.as_ref();
                if let Type::Path(path) = r#type {
                    if let Some(segment) = path.path.segments.iter().next() {
                        let ident = segment.ident.to_string();
                        borsh_serialize_impls.insert(ident);
                    }
                }
            }
        }
    }
    (error_name, account_serialize_impls, borsh_serialize_impls)
}

/// Parse accounts - structs which implement `AccountSerialize` and/or
/// `AnchorDeserialize`.
fn parse_accounts<'a>(
    ctx: &'a CrateContext,
    account_serialize_impls: &HashSet<String>,
) -> Vec<&'a syn::ItemStruct> {
    ctx.structs()
        .filter(|item_strct| {
            account_serialize_impls
                .get(&item_strct.ident.to_string())
                .is_some()
        })
        .collect()
}

/// Parse all structs implementing the `Accounts` trait.
fn parse_account_derives(ctx: &CrateContext) -> HashMap<String, AccountsStruct> {
    let accounts_impls: HashMap<String, ()> = ctx
        .impls()
        .filter_map(|i_impl| match &i_impl.trait_ {
            Some((_, path, _)) => {
                let mut segments = path.segments.iter();

                let segment_1 = segments.next();
                let segment_2 = segments.next();

                if let Some(segment_1) = segment_1 {
                    if let Some(segment_2) = segment_2 {
                        if segment_1.ident == "anchor_lang" && segment_2.ident == "Accounts" {
                            if let Type::Path(path) = i_impl.self_ty.as_ref() {
                                if let Some(segment) = path.path.segments.iter().next() {
                                    let ident = segment.ident.to_string();
                                    if [
                                        "IdlCreateAccounts",
                                        "IdlAccounts",
                                        "IdlResizeAccount",
                                        "IdlCreateBuffer",
                                        "IdlSetBuffer",
                                        "IdlCloseAccount",
                                    ]
                                    .iter()
                                    .any(|&s| s == ident)
                                    {
                                        return None;
                                    }
                                    return Some((ident, ()));
                                }
                            }
                        }
                    }
                }
                None
            }
            None => None,
        })
        .collect();
    ctx.structs()
        .filter_map(|i_strct| {
            if accounts_impls.get(&i_strct.ident.to_string()).is_some() {
                let strct = accounts::parse(i_strct).expect("Code not parseable");
                return Some((strct.ident.to_string(), strct));
            }
            None
        })
        .collect()
}

fn parse_consts(ctx: &CrateContext) -> Vec<&syn::ItemConst> {
    ctx.consts()
        .filter(|item_strct| {
            // TODO(vadorovsky): Find an another way to recognize consts for
            // which we should generate IDL.
            if item_strct.vis
                != syn::Visibility::Public(syn::VisPublic {
                    pub_token: Default::default(),
                })
            {
                return false;
            }
            true
        })
        .collect()
}

/// Parse all user defined types in the file.
fn parse_ty_defs(
    ctx: &CrateContext,
    account_serialize_impls: &HashSet<String>,
    borsh_serialize_impls: &HashSet<String>,
    no_docs: bool,
) -> Result<Vec<IdlTypeDefinition>> {
    ctx.structs()
        .filter_map(|item_strct| {
            // Only take serializable types
            if account_serialize_impls
                .get(&item_strct.ident.to_string())
                .is_none()
                && borsh_serialize_impls
                    .get(&item_strct.ident.to_string())
                    .is_none()
            {
                return None;
            }

            // Only take public types
            match &item_strct.vis {
                syn::Visibility::Public(_) => (),
                _ => return None,
            }

            parse_ty_def_strct(ctx, item_strct, no_docs).transpose()
        })
        .chain(ctx.enums().filter_map(|enm| {
            // Only take public types
            match &enm.vis {
                syn::Visibility::Public(_) => (),
                _ => return None,
            }

            parse_ty_def_enum(ctx, enm, no_docs).transpose()
        }))
        .collect()
}

/// Parse IDL type definition from a Rust struct.
fn parse_ty_def_strct(
    ctx: &CrateContext,
    item_strct: &syn::ItemStruct,
    no_docs: bool,
) -> Result<Option<IdlTypeDefinition>> {
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
            .filter_map(|f: &syn::Field| {
                let doc = if !no_docs {
                    docs::parse(&f.attrs)
                } else {
                    None
                };
                to_idl_type(ctx, &f.ty).transpose().map(|ty| {
                    ty.map(|ty| IdlField {
                        name: f.ident.as_ref().unwrap().to_string().to_mixed_case(),
                        docs: doc,
                        ty,
                    })
                })
            })
            .collect::<Result<Vec<IdlField>>>(),
        syn::Fields::Unnamed(_) => return Ok(None),
        _ => panic!("Empty structs are allowed."),
    };

    Some(fields.map(|fields| IdlTypeDefinition {
        name,
        docs: doc,
        ty: IdlTypeDefinitionTy::Struct { fields },
    }))
    .transpose()
}

fn parse_ty_def_enum(
    ctx: &CrateContext,
    item_enum: &syn::ItemEnum,
    no_docs: bool,
) -> Result<Option<IdlTypeDefinition>> {
    let name = item_enum.ident.to_string();
    let doc = if !no_docs {
        docs::parse(&item_enum.attrs)
    } else {
        None
    };
    let variants = item_enum
        .variants
        .iter()
        .map(|variant: &syn::Variant| {
            let name = variant.ident.to_string();
            let fields = match &variant.fields {
                syn::Fields::Unit => None,
                syn::Fields::Unnamed(fields) => {
                    let fields = fields
                        .unnamed
                        .iter()
                        .filter_map(|f| to_idl_type(ctx, &f.ty).transpose())
                        .collect::<Result<Vec<IdlType>>>()?;
                    Some(EnumFields::Tuple(fields))
                }
                syn::Fields::Named(fields) => {
                    let fields = fields
                        .named
                        .iter()
                        .filter_map(|f: &syn::Field| {
                            let name = f.ident.as_ref().unwrap().to_string();
                            let doc = if !no_docs {
                                docs::parse(&f.attrs)
                            } else {
                                None
                            };
                            to_idl_type(ctx, &f.ty).transpose().map(|ty| {
                                ty.map(|ty| IdlField {
                                    name,
                                    docs: doc,
                                    ty,
                                })
                            })
                        })
                        .collect::<Result<Vec<IdlField>>>()?;
                    Some(EnumFields::Named(fields))
                }
            };
            Ok(IdlEnumVariant { name, fields })
        })
        .collect::<Result<Vec<IdlEnumVariant>>>();
    match variants {
        Ok(variants) => Ok(Some(IdlTypeDefinition {
            name,
            docs: doc,
            ty: IdlTypeDefinitionTy::Enum { variants },
        })),
        Err(e) => Err(e),
    }
}

/// Parse all `use` tokens from the crate context and returns a map of imported
/// types and their path roots (external crates they're imported from).
fn parse_external_types(ctx: &CrateContext) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for use_ in ctx.uses() {
        // This should always be the case - the first `use` tree item should be
        // always be a path to either:
        // * external crate (case we are supporting right now)
        // * already imported module (TODO(vadorovsky): we don't support this
        //   case yet, but we should!)
        // * `self` or `crate` (we don't care about this case - we are always
        //   expanding macros, therefore we are already parsing the whole crate
        //   and we focus only on external imports)
        if let syn::UseTree::Path(p) = &use_.tree {
            let ident = &p.ident;
            if ident == "self" || ident == "crate" {
                continue;
            }
            parse_use_tree(&p.tree, &mut map, &ident.to_string())
        }
    }
    map
}

/// Parses subtress of `use` tokens and populates the `map` with types and
/// their base paths (external crates they're imported from).
fn parse_use_tree(use_tree: &syn::UseTree, map: &mut HashMap<String, String>, crate_name: &str) {
    match use_tree {
        syn::UseTree::Path(p) => {
            parse_use_tree(&p.tree, map, crate_name);
        }
        syn::UseTree::Name(n) => {
            map.insert(n.ident.to_string(), crate_name.to_owned());
        }
        syn::UseTree::Group(g) => {
            for use_tree in &g.items {
                parse_use_tree(use_tree, map, crate_name);
            }
        }
        // TODO(vadorovsky): Parse global imports too, just in case someone
        // is using globally imported type. Our current limitation is that we
        // only support types that are imported via `use` statement.
        // It will be REALLY tricky to do it in performant and feasible way
        // (the ideal solution would be doing so **without** parsing all
        // external crates, but not sure if it's possible).
        _ => {}
    }
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

fn to_idl_type(ctx: &CrateContext, ty: &syn::Type) -> Result<Option<IdlType>> {
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

    IdlType::from_str(&tts_string).map_err(|e| e.into())
}

// TODO parse other issues
fn to_idl_const(item: &ItemConst) -> Result<Option<IdlConst>> {
    let name = item.ident.to_string();

    if let Expr::Lit(ExprLit { lit, .. }) = &*item.expr {
        match lit {
            ByteStr(lit_byte_str) => {
                return Ok(Some(IdlConst {
                    name,
                    ty: IdlType::Bytes,
                    value: format!("{:?}", lit_byte_str.value()),
                }))
            }
            Byte(lit_byte) => {
                return Ok(Some(IdlConst {
                    name,
                    ty: IdlType::U8,
                    value: lit_byte.value().to_string(),
                }))
            }
            _ => (),
        }
    }

    Ok(
        IdlType::from_str(&item.ty.to_token_stream().to_string())?.map(|ty| IdlConst {
            name,
            ty,
            value: item.expr.to_token_stream().to_string().parse().unwrap(),
        }),
    )
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
                pda: pda::parse(ctx, accounts, acc, seeds_feature),
                relations: relations::parse(acc, seeds_feature),
            }),
        })
        .collect::<Vec<_>>()
}
