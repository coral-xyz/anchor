use crate::{
    CompositeField, Constraint, ConstraintAssociatedGroup, ConstraintBelongsTo,
    ConstraintExecutable, ConstraintGroup, ConstraintInit, ConstraintLiteral, ConstraintMut,
    ConstraintOwner, ConstraintRaw, ConstraintRentExempt, ConstraintSeeds, ConstraintSigner,
    ConstraintState, Field, Ty,
};
use quote::quote;

pub fn generate(f: &Field) -> proc_macro2::TokenStream {
    let checks: Vec<proc_macro2::TokenStream> = linearize(&f.constraints)
        .iter()
        .map(|c| generate_constraint(f, c))
        .collect();
    quote! {
        #(#checks)*
    }
}

pub fn generate_composite(f: &CompositeField) -> proc_macro2::TokenStream {
    let checks: Vec<proc_macro2::TokenStream> = linearize(&f.constraints)
        .iter()
        .filter_map(|c| match c {
            Constraint::Raw(_) => Some(c),
            Constraint::Literal(_) => Some(c),
            _ => panic!("Invariant violation: composite constraints can only be raw or literals"),
        })
        .map(|c| generate_constraint_composite(f, c))
        .collect();
    quote! {
        #(#checks)*
    }
}

// Linearizes the constraint group so that constraints with dependencies
// run after those without.
//
// The associated cosntraint should always be first since it may also create
// an account with a PDA.
pub fn linearize(c_group: &ConstraintGroup) -> Vec<Constraint> {
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
    } = c_group.clone();

    let mut constraints = Vec::new();

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

fn generate_constraint(f: &Field, c: &Constraint) -> proc_macro2::TokenStream {
    match c {
        Constraint::Init(c) => generate_constraint_init(f, c),
        Constraint::Mut(c) => generate_constraint_mut(f, c),
        Constraint::BelongsTo(c) => generate_constraint_belongs_to(f, c),
        Constraint::Signer(c) => generate_constraint_signer(f, c),
        Constraint::Literal(c) => generate_constraint_literal(c),
        Constraint::Raw(c) => generate_constraint_raw(c),
        Constraint::Owner(c) => generate_constraint_owner(f, c),
        Constraint::RentExempt(c) => generate_constraint_rent_exempt(f, c),
        Constraint::Seeds(c) => generate_constraint_seeds(f, c),
        Constraint::Executable(c) => generate_constraint_executable(f, c),
        Constraint::State(c) => generate_constraint_state(f, c),
        Constraint::AssociatedGroup(c) => generate_constraint_associated(f, c),
    }
}

fn generate_constraint_composite(_f: &CompositeField, c: &Constraint) -> proc_macro2::TokenStream {
    match c {
        Constraint::Raw(c) => generate_constraint_raw(c),
        Constraint::Literal(c) => generate_constraint_literal(c),
        _ => panic!("Invariant violation"),
    }
}

pub fn generate_constraint_init(_f: &Field, _c: &ConstraintInit) -> proc_macro2::TokenStream {
    quote! {}
}

pub fn generate_constraint_mut(f: &Field, _c: &ConstraintMut) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    quote! {
        if !#ident.to_account_info().is_writable {
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(36)); // todo: error codes
        }
    }
}

pub fn generate_constraint_belongs_to(
    f: &Field,
    c: &ConstraintBelongsTo,
) -> proc_macro2::TokenStream {
    let target = c.join_target.clone();
    let ident = &f.ident;
    let field = match &f.ty {
        Ty::Loader(_) => quote! {#ident.load()?},
        _ => quote! {#ident},
    };
    quote! {
        if &#field.#target != #target.to_account_info().key {
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(1)); // todo: error codes
        }
    }
}

pub fn generate_constraint_signer(f: &Field, _c: &ConstraintSigner) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let info = match f.ty {
        Ty::AccountInfo => quote! { #ident },
        Ty::ProgramAccount(_) => quote! { #ident.to_account_info() },
        _ => panic!("Invalid syntax: signer cannot be specified."),
    };
    quote! {
        // Don't enforce on CPI, since usually a program is signing and so
        // the `try_accounts` deserializatoin will fail *if* the one
        // tries to manually invoke it.
        //
        // This check will be performed on the other end of the invocation.
        if cfg!(not(feature = "cpi")) {
            if !#info.to_account_info().is_signer {
                return Err(anchor_lang::solana_program::program_error::ProgramError::MissingRequiredSignature);
            }
        }
    }
}

pub fn generate_constraint_literal(c: &ConstraintLiteral) -> proc_macro2::TokenStream {
    let lit: proc_macro2::TokenStream = {
        let lit = &c.lit;
        let lit_ts: proc_macro2::TokenStream = quote! {#lit};
        lit_ts.to_string().replace("\"", "").parse().unwrap()
    };
    quote! {
        if !(#lit) {
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(1)); // todo: error codes
        }
    }
}

pub fn generate_constraint_raw(c: &ConstraintRaw) -> proc_macro2::TokenStream {
    let raw = &c.raw;
    quote! {
        if !(#raw) {
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(14)); // todo: error codes
        }
    }
}

pub fn generate_constraint_owner(f: &Field, c: &ConstraintOwner) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let owner_target = c.owner_target.clone();
    quote! {
        if #ident.to_account_info().owner != #owner_target.to_account_info().key {
            return Err(ProgramError::Custom(76)); // todo: proper error.
        }
    }
}

pub fn generate_constraint_rent_exempt(
    f: &Field,
    c: &ConstraintRentExempt,
) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let info = match f.ty {
        Ty::AccountInfo => quote! { #ident },
        Ty::ProgramAccount(_) => quote! { #ident.to_account_info() },
        Ty::Loader(_) => quote! { #ident.to_account_info() },
        _ => panic!("Invalid syntax: rent exemption cannot be specified."),
    };
    match c {
        ConstraintRentExempt::Skip => quote! {},
        ConstraintRentExempt::Enforce => quote! {
            if !rent.is_exempt(#info.lamports(), #info.try_data_len()?) {
                return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(2)); // todo: error codes
            }
        },
    }
}

pub fn generate_constraint_seeds(f: &Field, c: &ConstraintSeeds) -> proc_macro2::TokenStream {
    let name = &f.ident;
    let seeds = &c.seeds;
    quote! {
        let program_signer = Pubkey::create_program_address(
            &[#seeds],
            program_id,
        ).map_err(|_| anchor_lang::solana_program::program_error::ProgramError::Custom(1))?; // todo
        if #name.to_account_info().key != &program_signer {
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(1)); // todo
        }
    }
}

pub fn generate_constraint_executable(
    f: &Field,
    _c: &ConstraintExecutable,
) -> proc_macro2::TokenStream {
    let name = &f.ident;
    quote! {
        if !#name.to_account_info().executable {
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(5)) // todo
        }
    }
}

pub fn generate_constraint_state(f: &Field, c: &ConstraintState) -> proc_macro2::TokenStream {
    let program_target = c.program_target.clone();
    let ident = &f.ident;
    let account_ty = match &f.ty {
        Ty::CpiState(ty) => &ty.account_ident,
        _ => panic!("Invalid state constraint"),
    };
    quote! {
        // Checks the given state account is the canonical state account for
        // the target program.
        if #ident.to_account_info().key != &anchor_lang::CpiState::<#account_ty>::address(#program_target.to_account_info().key) {
            return Err(ProgramError::Custom(1)); // todo: proper error.
        }
        if #ident.to_account_info().owner != #program_target.to_account_info().key {
            return Err(ProgramError::Custom(1)); // todo: proper error.
        }
    }
}

pub fn generate_constraint_associated(
    f: &Field,
    c: &ConstraintAssociatedGroup,
) -> proc_macro2::TokenStream {
    if c.is_init {
        generate_constraint_associated_init(f, c)
    } else {
        generate_constraint_associated_seeds(f, c)
    }
}

pub fn generate_constraint_associated_init(
    f: &Field,
    c: &ConstraintAssociatedGroup,
) -> proc_macro2::TokenStream {
    let associated_target = c.associated_target.clone();
    let field = &f.ident;
    let (account_ty, is_zero_copy) = match &f.ty {
        Ty::ProgramAccount(ty) => (&ty.account_ident, false),
        Ty::Loader(ty) => (&ty.account_ident, true),
        _ => panic!("Invalid associated constraint"),
    };

    let space = match &c.space {
        // If no explicit space param was given, serialize the type to bytes
        // and take the length (with +8 for the discriminator.)
        None => match is_zero_copy {
            false => {
                quote! {
                    let space = 8 + #account_ty::default().try_to_vec().unwrap().len();
                }
            }
            true => {
                quote! {
                    let space = 8 + anchor_lang::__private::bytemuck::bytes_of(&#account_ty::default()).len();
                }
            }
        },
        // Explicit account size given. Use it.
        Some(s) => quote! {
            let space = #s;
        },
    };

    let payer = match &c.payer {
        None => quote! {
            let payer = #associated_target.to_account_info();
        },
        Some(p) => quote! {
            let payer = #p.to_account_info();
        },
    };

    let associated_pubkey_and_nonce = generate_associated_pubkey(f, c);

    let seeds_with_nonce = match c.associated_seeds.len() {
        0 => quote! {
            [
                &b"anchor"[..],
                #associated_target.to_account_info().key.as_ref(),
                &[nonce],
            ]
        },
        _ => {
            let seeds = to_seeds_tts(&c.associated_seeds);
            quote! {
                [
                    &b"anchor"[..],
                    #associated_target.to_account_info().key.as_ref(),
                    #seeds
                    &[nonce],
                ]
            }
        }
    };

    let account_wrapper_ty = match is_zero_copy {
        false => quote! {
            anchor_lang::ProgramAccount
        },
        true => quote! {
            anchor_lang::Loader
        },
    };
    let nonce_assignment = match is_zero_copy {
        false => quote! {},
        // Zero copy is not deserialized, so the data must be lazy loaded.
        true => quote! {
            .load_init()?
        },
    };

    quote! {
        let #field: #account_wrapper_ty<#account_ty> = {
            #space
            #payer

            #associated_pubkey_and_nonce

            if &__associated_field != #field.key {
                return Err(ProgramError::Custom(45)); // todo: proper error.
            }
            let lamports = rent.minimum_balance(space);
            let ix = anchor_lang::solana_program::system_instruction::create_account(
                payer.key,
                #field.key,
                lamports,
                space as u64,
                program_id,
            );

            let seeds = #seeds_with_nonce;
            let signer = &[&seeds[..]];
            anchor_lang::solana_program::program::invoke_signed(
                &ix,
                &[

                    #field.clone(),
                    payer.clone(),
                    system_program.clone(),
                ],
                signer,
            ).map_err(|e| {
                anchor_lang::solana_program::msg!("Unable to create associated account");
                e
            })?;
            // For now, we assume all accounts created with the `associated`
            // attribute have a `nonce` field in their account.
            let mut pa: #account_wrapper_ty<#account_ty> = #account_wrapper_ty::try_from_init(
                &#field,
            )?;
            pa#nonce_assignment.__nonce = nonce;
            pa
        };
    }
}

pub fn generate_constraint_associated_seeds(
    f: &Field,
    c: &ConstraintAssociatedGroup,
) -> proc_macro2::TokenStream {
    let generated_associated_pubkey_and_nonce = generate_associated_pubkey(f, c);
    let name = &f.ident;
    quote! {
        #generated_associated_pubkey_and_nonce
        if #name.to_account_info().key != &__associated_field {
            // TODO: proper error.
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(45));
        }
    }
}

pub fn generate_associated_pubkey(
    _f: &Field,
    c: &ConstraintAssociatedGroup,
) -> proc_macro2::TokenStream {
    let associated_target = c.associated_target.clone();
    let seeds_no_nonce = match c.associated_seeds.len() {
        0 => quote! {
            [
                &b"anchor"[..],
                #associated_target.to_account_info().key.as_ref(),
            ]
        },
        _ => {
            let seeds = to_seeds_tts(&c.associated_seeds);
            quote! {
                [
                    &b"anchor"[..],
                    #associated_target.to_account_info().key.as_ref(),
                    #seeds
                ]
            }
        }
    };
    quote! {
        let (__associated_field, nonce) = Pubkey::find_program_address(
            &#seeds_no_nonce,
            program_id,
        );
    }
}

// Returns the inner part of the seeds slice as a token stream.
fn to_seeds_tts(seeds: &[syn::Ident]) -> proc_macro2::TokenStream {
    assert!(seeds.len() > 0);
    let seed_0 = &seeds[0];
    let mut tts = quote! {
        #seed_0.to_account_info().key.as_ref(),
    };
    for seed in &seeds[1..] {
        tts = quote! {
            #tts
            #seed.to_account_info().key.as_ref(),
        };
    }
    tts
}
