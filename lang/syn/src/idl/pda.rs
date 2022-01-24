use crate::idl::*;
use crate::parser;
use crate::parser::context::CrateContext;
use crate::ConstraintSeedsGroup;
use crate::{AccountsStruct, Field};
use std::collections::HashMap;
use std::str::FromStr;
use syn::Expr;

// Parses a seeds constraint, extracting the IdlSeed types.
//
// Note: This implementation makes assumptions about the types that can be used
//       (e.g., no program-defined function calls in seeds).
//
//       This probably doesn't cover all cases. If you see a warning log, you
//       can add a new case here. In the worst case, we miss a seed and
//       the parser will treat the given seeds as empty and so clients will
//       simply fail to automatically populate the PDA accounts.
//
// Seed Assumptions: Seeds must be of one of the following forms:
//
// - instruction argument.
// - account context field pubkey.
// - account data, where the account is defined in the current program.
//   We make an exception for the SPL token program, since it is so common
//   and sometimes convenient to use fields as a seed (e.g. Auction house
//   program). In the case of nested structs/account data, all nested structs
//   must be defined in the current program as well.
// - byte string literal (e.g. b"MY_SEED").
// - byte string literal constant  (e.g. `pub const MY_SEED: [u8; 2] = *b"hi";`).
// - array constants.
//
pub fn parse(
    ctx: &CrateContext,
    accounts: &AccountsStruct,
    acc: &Field,
    seeds_feature: bool,
) -> Option<IdlPda> {
    if !seeds_feature {
        return None;
    }

    let pda_parser = PdaParser::new(ctx, accounts);
    acc.constraints
        .seeds
        .as_ref()
        .map(|s| pda_parser.parse(s))
        .unwrap_or(None)
}

struct PdaParser<'a> {
    ctx: &'a CrateContext,
    // Accounts context.
    accounts: &'a AccountsStruct,
    // Maps var name to var type. These are the instruction arguments in a
    // given accounts context.
    ix_args: HashMap<String, String>,
    // Constants available in the crate.
    const_names: Vec<String>,
    // All field names of the accounts in the accounts context.
    account_field_names: Vec<String>,
}

impl<'a> PdaParser<'a> {
    fn new(ctx: &'a CrateContext, accounts: &'a AccountsStruct) -> Self {
        // All the available sources of seeds.
        let ix_args = accounts.instruction_args().unwrap_or_default();
        let const_names: Vec<String> = ctx.consts().map(|c| c.ident.to_string()).collect();
        let account_field_names = accounts.field_names();
        Self {
            ctx,
            accounts,
            ix_args,
            const_names,
            account_field_names,
        }
    }

    fn parse(&self, seeds_grp: &ConstraintSeedsGroup) -> Option<IdlPda> {
        // Extract the idl seed types from the constraints.
        let seeds = seeds_grp
            .seeds
            .iter()
            .map(|seed| {
                parse_seed(
                    self.ctx,
                    self.accounts,
                    &self.ix_args,
                    &self.const_names,
                    &self.account_field_names,
                    seed,
                )
            })
            .collect::<Option<Vec<_>>>()?;

        // Parse the program id from the constraints.
        let program_id = seeds_grp
            .program_seed
            .as_ref()
            .map(|pid| {
                parse_seed(
                    self.ctx,
                    self.accounts,
                    &self.ix_args,
                    &self.const_names,
                    &self.account_field_names,
                    pid,
                )
            })
            .unwrap_or_default();

        // Done.
        Some(IdlPda { seeds, program_id })
    }
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
            // Parse the seed components.
            let seed_path = parse_seed_path(seed)?;

            match seed_path {
                // Instruction argument.
                _ if ix_args.contains_key(&seed_path.name()) => {
                    let idl_ty = IdlType::from_str(ix_args.get(&seed_path.name()).unwrap()).ok()?;
                    Some(IdlSeed::Arg(IdlSeedArg {
                        ty: idl_ty,
                        path: seed_path.path(),
                    }))
                }
                // Constant.
                _ if const_names.contains(&seed_path.name()) => {
                    // Pull in the constant value directly into the IDL.
                    assert!(seed_path.components().is_empty());
                    let const_item = ctx.consts().find(|c| c.ident == seed_path.name()).unwrap();
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
                _ if account_field_names.contains(&seed_path.name()) => {
                    Some(IdlSeed::Account(IdlSeedAccount {
                        ty: parse_seed_account_field_ty(
                            ctx,
                            accounts,
                            seed_path.name().to_string(),
                            seed_path.components(),
                        )?,
                        path: seed_path.path(),
                        account: parse_seed_account_ty(ctx, accounts, seed_path.name()),
                    }))
                }
                // String literal.
                _ if seed_path.components().is_empty() && seed_path.name().contains('"') => {
                    let mut var_name = seed_path.name().to_string();
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
                _ => {
                    println!("WARNING: unexpected seed category for var: {:?}", seed_path);
                    None
                }
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

// SeedPath represents the deconstructed syntax of a single pda seed,
// consisting of a variable name and a vec of all the sub fields accessed
// on that variable name. For example, if a seed is `my_field.my_data.as_ref()`,
// then the field name is `my_field` and the vec of sub fields is `[my_data]`.
#[derive(Debug)]
struct SeedPath(String, Vec<String>);

impl SeedPath {
    fn name(&self) -> String {
        self.0.clone()
    }

    // Full path to the data this seed represents.
    fn path(&self) -> String {
        match self.1.len() {
            0 => self.0.clone(),
            _ => format!("{}.{}", self.name(), self.components().join(".")),
        }
    }

    // All path components for the subfields accessed on this seed.
    fn components(&self) -> &[String] {
        &self.1
    }
}

// Extracts the seed path from a single seed expression.
fn parse_seed_path(seed: &Expr) -> Option<SeedPath> {
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
    while !components.is_empty() {
        let c = components.remove(0);
        if c.contains("()") {
            break;
        }
        path.push(c.to_string());
    }
    if path.len() == 1 && (path[0] == "key" || path[0] == "key()") {
        path = Vec::new();
    }

    Some(SeedPath(name, path))
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
