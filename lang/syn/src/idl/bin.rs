use crate::idl::*;
use crate::parser::context::CrateContext;
use crate::parser::{self, accounts, docs, error, program};
use crate::Ty;
use crate::{AccountField, AccountsStruct, StateIx};
use anyhow::Result;
use heck::MixedCase;
use quote::{ToTokens, quote, format_ident};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const DERIVE_NAME: &str = "Accounts";
// TODO: share this with `anchor_lang` crate.
const ERROR_CODE_OFFSET: u32 = 6000;

// Generate the source of the idl binary
pub fn gen_src(
    filename: impl AsRef<Path>,
    version: String,
    seeds_feature: bool,
    no_docs: bool,
    safety_checks: bool,
) -> Result<Option<String>> {
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

    let account_struct = accs.get("Initialize").unwrap();
    let accounts = account_struct.fields.iter().map(|acc: &AccountField| {
        match acc {
            AccountField::CompositeField(_) => panic!("TODO"),
            AccountField::Field(acc) => {
                let name = acc.ident.to_string();
                let is_mut = acc.constraints.is_mutable();
                let is_signer = match acc.ty {
                    Ty::Signer => true,
                    _ => acc.constraints.is_signer()
                };

                let mut fields = vec![
                    quote!("name": #name),
                    quote!("isMut": #is_mut),
                    quote!("isSigner": #is_signer),
                    // TODO: docs
                ];
                
                // pubkey
                // TODO: also handle `Sysvar` and `address = <>` constraint
                let pubkey = match &acc.ty {
                    // transform from `Program<'info, SomeType>` to `SomeType::id().to_string()`
                    Ty::Program(program) => program.account_type_path.path.get_ident().map(|i| quote!{#i::id().to_string()}),
                    _ => None
                };
                pubkey.map(|pubkey| fields.push(quote!{"pubkey": #pubkey}));

                // seeds
                let seeds: Option<Vec<proc_macro2::TokenStream>> = acc.constraints.seeds.as_ref().map(|seeds| {
                    // TODO: cover the cases when seed expression referencess instruction args or accounts
                    seeds.seeds.iter().map(|seed| quote!{
                        {
                            "kind": "const",
                            "type": "base58",
                            "value": bs58::encode(#seed).into_string()
                        }
                    }).collect()
                });
                // TODO handle `seeds::program = <>` constraint
                seeds.map(|seeds| fields.push(quote!("pda": {
                    "seeds": [#(#seeds),*]
                })));


                quote!{
                    {
                       #(#fields),*
                    }
                }
            }
        }
    });

    
    let ret = quote!{
        use anchor_lang::prelude::*;
        use std::str::FromStr;

        const MY_SEED_U64: u64 = 3;

        fn main() {
            let instructions = serde_json::json!({
                "instructions": [
                    {
                        "name": "initialize",
                        "accounts": [#(#accounts),*],
                        "args": []
                    }
                ]
            });

            println!("{}", serde_json::to_string_pretty(&instructions).unwrap());
        }
    };
    
    Ok(Some(format!("{}", ret)))
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