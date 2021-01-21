extern crate proc_macro;

use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn constructor(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let seeds: proc_macro2::TokenStream = args.to_string().parse().unwrap();

    let item_fn = parse_macro_input!(input as syn::ItemFn);

    let fn_vis = item_fn.vis;
    let fn_sig = item_fn.sig;
    let fn_name = &fn_sig.ident;
    let fn_block = item_fn.block;
    let mut fn_inputs_iter = fn_sig.inputs.iter();
    fn_inputs_iter
        .next()
        .expect("Must have one argument at least");
    let fn_inputs = fn_inputs_iter.map(|i| quote! {#i}).collect::<Vec<_>>();

    let fn_args: Vec<Box<syn::Pat>> = {
        let mut iter = fn_sig.inputs.iter();
        iter.next().expect("Must have one argument");
        iter.map(|arg: &syn::FnArg| match arg {
            syn::FnArg::Typed(pat_ty) => pat_ty.pat.clone(),
            _ => panic!("Invalid syntax"),
        })
        .collect()
    };
    let fn_stmts = fn_block.stmts;

    let account_ty = {
        let fn_arg: &syn::FnArg = fn_sig
            .inputs
            .first()
            .expect("Constructors must have an account argument");
        let ty = match fn_arg {
            syn::FnArg::Typed(pat_ty) => *pat_ty.ty.clone(),
            _ => panic!("Invalid syntax"),
        };
        let generic_args = match ty {
            syn::Type::Reference(ty_ref) => match *ty_ref.elem {
                syn::Type::Path(path) => path
                    .path
                    .segments
                    .last()
                    .expect("Invalid syntax")
                    .arguments
                    .clone(),
                _ => panic!("Invalid syntax"),
            },
            _ => panic!("Invalid syntax"),
        };

        let ty = match generic_args {
            syn::PathArguments::AngleBracketed(ref angle_b) => {
                let args = angle_b.args.last().clone();
                let ty = args.expect("invalid syntax");
                ty
            }
            _ => panic!("invalid syntax"),
        };
        let r = match &ty {
            syn::GenericArgument::Type(ty) => match ty {
                syn::Type::Path(path) => path.path.segments.last().expect("Invalid syntax"),
                _ => panic!("Invalid syntax"),
            },
            _ => panic!("Invalid syntax"),
        };
        r.ident.clone()
    };

    let mod_name: proc_macro2::TokenStream = format!("__private_{}", fn_name.to_string())
        .parse()
        .unwrap();
    // await program.constructor.registry();
    proc_macro::TokenStream::from(quote! {
        #fn_vis fn #fn_name(ctx: Context<Constructor>, #(#fn_inputs),*) -> Result<(), Error>{
            let from = ctx.accounts.from.key;
            let (base, nonce) = Pubkey::find_program_address(&[], ctx.accounts.program.key);
            let seed = #seeds;
            let owner = ctx.accounts.program.key;
            let to = Pubkey::create_with_seed(&base, seed, owner).unwrap();
            let space = #account_ty::SIZE;
            let lamports = ctx.accounts.rent.minimum_balance(space);
            let seeds = &[&[nonce][..]];

            // Create the new program owned account (from within the program).
            let ix = anchor_lang::solana_program::system_instruction::create_account_with_seed(
                from,
                &to,
                &base,
                seed,
                lamports,
                space as u64,
                owner,
            );
            anchor_lang::solana_program::program::invoke_signed(
                &ix,
                &[
                    ctx.accounts.from.clone(),
                    ctx.accounts.to.clone(),
                    ctx.accounts.base.clone(),
                    ctx.accounts.system_program.clone(),
                ],
                &[seeds],
            )?;

            // Deserialize the newly created account.
            let mut r: ProgramAccount<#account_ty> = ProgramAccount::try_from_init(&ctx.accounts.to)?;

            // Run the user's defined function.
            #mod_name::#fn_name(&mut r, #(#fn_args),*)?;

            // Call `exit` to persist the changes made by the user's function.
            r.exit(ctx.program_id)?;

            Ok(())
        }

        mod #mod_name {
            use super::*;
            pub #fn_sig {
                #(#fn_stmts)*
            }
        }
    })
}

/*
    // Global ctor for the entire program. We use a deterministic address
    // to store global state, i.e., the address of the lockup program.
    pub fn constructor(ctx: Context<Constructor>, lockup_program: Pubkey) -> Result<(), Error> {
        let from = ctx.accounts.from.key;
        let (base, nonce) = Pubkey::find_program_address(&[], ctx.accounts.program.key);
        let seed = "v1";
        let owner = ctx.accounts.program.key;
        let to = Pubkey::create_with_seed(&base, seed, owner).unwrap();
        let space = Registry::SIZE;
        let lamports = ctx.accounts.rent.minimum_balance(space);

        let seeds = &[&[nonce][..]];

        let ix = anchor_lang::solana_program::system_instruction::create_account_with_seed(
            from,
            &to,
            &base,
            seed,
            lamports,
            space as u64,
            owner,
        );
        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.from.clone(),
                ctx.accounts.to.clone(),
                ctx.accounts.base.clone(),
                ctx.accounts.system_program.clone(),
            ],
            &[seeds],
        )?;

        let mut registry: ProgramAccount<Registry> =
            ProgramAccount::try_from_init(&ctx.accounts.to)?;
        registry.lockup_program = lockup_program;
        registry.exit(ctx.program_id)?;

        Ok(())
    }

*/
