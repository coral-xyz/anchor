use crate::idl::*;
use crate::parser::accounts;
use crate::parser::program;
use crate::AccountsStruct;
use anyhow::Result;
use heck::MixedCase;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

static DERIVE_NAME: &'static str = "Accounts";

// Parse an entire interface file.
pub fn parse(filename: impl AsRef<Path>) -> Result<Idl> {
    let mut file = File::open(&filename)?;

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

    let f = syn::parse_file(&src).expect("Unable to parse file");

    let p = program::parse(parse_program_mod(&f));

    let accs = parse_accounts(&f);

    let acc_names = {
        let mut acc_names = HashSet::new();

        for accs_strct in accs.values() {
            for a in accs_strct.account_tys() {
                acc_names.insert(a);
            }
        }

        acc_names
    };

    let instructions = p
        .rpcs
        .iter()
        .map(|rpc| {
            let args = rpc
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
            let accounts_strct = accs.get(&rpc.anchor_ident.to_string()).unwrap();
            let accounts = accounts_strct
                .fields
                .iter()
                .map(|acc| IdlAccount {
                    name: acc.ident.to_string().to_mixed_case(),
                    is_mut: acc.is_mut,
                    is_signer: acc.is_signer,
                })
                .collect::<Vec<_>>();
            IdlInstruction {
                name: rpc.ident.to_string().to_mixed_case(),
                accounts,
                args,
            }
        })
        .collect::<Vec<_>>();

    // All user defined types.
    let mut accounts = vec![];
    let mut types = vec![];
    let ty_defs = parse_ty_defs(&f)?;
    for ty_def in ty_defs {
        if acc_names.contains(&ty_def.name) {
            accounts.push(ty_def);
        } else {
            types.push(ty_def);
        }
    }

    Ok(Idl {
        version: "0.0.0".to_string(),
        name: p.name.to_string(),
        instructions,
        types,
        accounts,
        metadata: None,
    })
}

// Parse the main program mod.
fn parse_program_mod(f: &syn::File) -> syn::ItemMod {
    let mods = f
        .items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Mod(item_mod) => {
                let mods = item_mod
                    .attrs
                    .iter()
                    .filter_map(|attr| {
                        let segment = attr.path.segments.last().unwrap();
                        if segment.ident.to_string() == "program" {
                            return Some(attr);
                        }
                        None
                    })
                    .collect::<Vec<_>>();
                if mods.len() != 1 {
                    panic!("invalid program attribute");
                }
                Some(item_mod)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(mods.len() == 1);
    mods[0].clone()
}

// Parse all structs deriving the `Accounts` macro.
fn parse_accounts(f: &syn::File) -> HashMap<String, AccountsStruct> {
    f.items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Struct(i_strct) => {
                for attr in &i_strct.attrs {
                    if attr.tokens.to_string().contains(DERIVE_NAME) {
                        let strct = accounts::parse(i_strct);
                        return Some((strct.ident.to_string(), strct));
                    }
                }
                None
            }
            _ => None,
        })
        .collect()
}

// Parse all user defined types in the file.
fn parse_ty_defs(f: &syn::File) -> Result<Vec<IdlTypeDef>> {
    f.items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Struct(item_strct) => {
                for attr in &item_strct.attrs {
                    if attr.tokens.to_string().contains(DERIVE_NAME) {
                        return None;
                    }
                }
                if let syn::Visibility::Public(_) = &item_strct.vis {
                    let name = item_strct.ident.to_string();
                    let fields = match &item_strct.fields {
                        syn::Fields::Named(fields) => fields
                            .named
                            .iter()
                            .map(|f| {
                                let mut tts = proc_macro2::TokenStream::new();
                                f.ty.to_tokens(&mut tts);
                                Ok(IdlField {
                                    name: f.ident.as_ref().unwrap().to_string().to_mixed_case(),
                                    ty: tts.to_string().parse()?,
                                })
                            })
                            .collect::<Result<Vec<IdlField>>>(),
                        _ => panic!("Only named structs are allowed."),
                    };

                    return Some(fields.map(|fields| IdlTypeDef {
                        name,
                        ty: IdlTypeDefTy::Struct { fields },
                    }));
                }
                None
            }
            _ => None,
        })
        .collect()
}
