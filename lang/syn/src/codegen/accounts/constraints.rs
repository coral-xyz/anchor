use crate::*;
use proc_macro2_diagnostics::SpanDiagnosticExt;
use quote::quote;
use syn::Expr;

pub fn generate(f: &Field) -> proc_macro2::TokenStream {
    let constraints = linearize(&f.constraints);

    let rent = constraints
        .iter()
        .any(|c| matches!(c, Constraint::RentExempt(ConstraintRentExempt::Enforce)))
        .then(|| quote! { let __anchor_rent = Rent::get()?; })
        .unwrap_or_else(|| quote! {});

    let checks: Vec<proc_macro2::TokenStream> = constraints
        .iter()
        .map(|c| generate_constraint(f, c))
        .collect();

    quote! {
        #rent
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
pub fn linearize(c_group: &ConstraintGroup) -> Vec<Constraint> {
    let ConstraintGroup {
        init,
        zeroed,
        mutable,
        signer,
        has_one,
        literal,
        raw,
        owner,
        rent_exempt,
        seeds,
        executable,
        state,
        close,
        address,
    } = c_group.clone();

    let mut constraints = Vec::new();

    if let Some(c) = zeroed {
        constraints.push(Constraint::Zeroed(c));
    }
    if let Some(c) = init {
        constraints.push(Constraint::Init(c));
    }
    if let Some(c) = seeds {
        constraints.push(Constraint::Seeds(c));
    }
    if let Some(c) = mutable {
        constraints.push(Constraint::Mut(c));
    }
    if let Some(c) = signer {
        constraints.push(Constraint::Signer(c));
    }
    constraints.append(&mut has_one.into_iter().map(Constraint::HasOne).collect());
    constraints.append(&mut literal.into_iter().map(Constraint::Literal).collect());
    constraints.append(&mut raw.into_iter().map(Constraint::Raw).collect());
    if let Some(c) = owner {
        constraints.push(Constraint::Owner(c));
    }
    if let Some(c) = rent_exempt {
        constraints.push(Constraint::RentExempt(c));
    }
    if let Some(c) = executable {
        constraints.push(Constraint::Executable(c));
    }
    if let Some(c) = state {
        constraints.push(Constraint::State(c));
    }
    if let Some(c) = close {
        constraints.push(Constraint::Close(c));
    }
    if let Some(c) = address {
        constraints.push(Constraint::Address(c));
    }
    constraints
}

fn generate_constraint(f: &Field, c: &Constraint) -> proc_macro2::TokenStream {
    match c {
        Constraint::Init(c) => generate_constraint_init(f, c),
        Constraint::Zeroed(c) => generate_constraint_zeroed(f, c),
        Constraint::Mut(c) => generate_constraint_mut(f, c),
        Constraint::HasOne(c) => generate_constraint_has_one(f, c),
        Constraint::Signer(c) => generate_constraint_signer(f, c),
        Constraint::Literal(c) => generate_constraint_literal(c),
        Constraint::Raw(c) => generate_constraint_raw(c),
        Constraint::Owner(c) => generate_constraint_owner(f, c),
        Constraint::RentExempt(c) => generate_constraint_rent_exempt(f, c),
        Constraint::Seeds(c) => generate_constraint_seeds(f, c),
        Constraint::Executable(c) => generate_constraint_executable(f, c),
        Constraint::State(c) => generate_constraint_state(f, c),
        Constraint::Close(c) => generate_constraint_close(f, c),
        Constraint::Address(c) => generate_constraint_address(f, c),
    }
}

fn generate_constraint_composite(_f: &CompositeField, c: &Constraint) -> proc_macro2::TokenStream {
    match c {
        Constraint::Raw(c) => generate_constraint_raw(c),
        Constraint::Literal(c) => generate_constraint_literal(c),
        _ => panic!("Invariant violation"),
    }
}

fn generate_constraint_address(f: &Field, c: &ConstraintAddress) -> proc_macro2::TokenStream {
    let field = &f.ident;
    let addr = &c.address;
    quote! {
        if #field.to_account_info().key != &#addr {
            return Err(anchor_lang::__private::ErrorCode::ConstraintAddress.into());
        }
    }
}

pub fn generate_constraint_init(f: &Field, c: &ConstraintInitGroup) -> proc_macro2::TokenStream {
    generate_constraint_init_group(f, c)
}

pub fn generate_constraint_zeroed(f: &Field, _c: &ConstraintZeroed) -> proc_macro2::TokenStream {
    let field = &f.ident;
    let (account_ty, account_wrapper_ty, _) = parse_ty(f);
    quote! {
        let #field: #account_wrapper_ty<#account_ty> = {
            let mut __data: &[u8] = &#field.try_borrow_data()?;
            let mut __disc_bytes = [0u8; 8];
            __disc_bytes.copy_from_slice(&__data[..8]);
            let __discriminator = u64::from_le_bytes(__disc_bytes);
            if __discriminator != 0 {
                return Err(anchor_lang::__private::ErrorCode::ConstraintZero.into());
            }
            #account_wrapper_ty::try_from_unchecked(
                &#field,
            )?
        };
    }
}

pub fn generate_constraint_close(f: &Field, c: &ConstraintClose) -> proc_macro2::TokenStream {
    let field = &f.ident;
    let target = &c.sol_dest;
    quote! {
        if #field.to_account_info().key == #target.to_account_info().key {
            return Err(anchor_lang::__private::ErrorCode::ConstraintClose.into());
        }
    }
}

pub fn generate_constraint_mut(f: &Field, _c: &ConstraintMut) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    quote! {
        if !#ident.to_account_info().is_writable {
            return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
        }
    }
}

pub fn generate_constraint_has_one(f: &Field, c: &ConstraintHasOne) -> proc_macro2::TokenStream {
    let target = c.join_target.clone();
    let ident = &f.ident;
    let field = match &f.ty {
        Ty::Loader(_) => quote! {#ident.load()?},
        _ => quote! {#ident},
    };
    quote! {
        if &#field.#target != #target.to_account_info().key {
            return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
        }
    }
}

pub fn generate_constraint_signer(f: &Field, _c: &ConstraintSigner) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let info = match f.ty {
        Ty::AccountInfo => quote! { #ident },
        Ty::ProgramAccount(_) => quote! { #ident.to_account_info() },
        Ty::Loader(_) => quote! { #ident.to_account_info() },
        Ty::CpiAccount(_) => quote! { #ident.to_account_info() },
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
                return Err(anchor_lang::__private::ErrorCode::ConstraintSigner.into());
            }
        }
    }
}

pub fn generate_constraint_literal(c: &ConstraintLiteral) -> proc_macro2::TokenStream {
    let lit: proc_macro2::TokenStream = {
        let lit = &c.lit;
        let constraint = lit.value().replace("\"", "");
        let message = format!(
            "Deprecated. Should be used with constraint: #[account(constraint = {})]",
            constraint,
        );
        lit.span().warning(message).emit_as_item_tokens();
        constraint.parse().unwrap()
    };
    quote! {
        if !(#lit) {
            return Err(anchor_lang::__private::ErrorCode::Deprecated.into());
        }
    }
}

pub fn generate_constraint_raw(c: &ConstraintRaw) -> proc_macro2::TokenStream {
    let raw = &c.raw;
    quote! {
        if !(#raw) {
            return Err(anchor_lang::__private::ErrorCode::ConstraintRaw.into());
        }
    }
}

pub fn generate_constraint_owner(f: &Field, c: &ConstraintOwner) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let owner_target = c.owner_target.clone();
    quote! {
        if #ident.to_account_info().owner != #owner_target.to_account_info().key {
            return Err(anchor_lang::__private::ErrorCode::ConstraintOwner.into());
        }
    }
}

pub fn generate_constraint_rent_exempt(
    f: &Field,
    c: &ConstraintRentExempt,
) -> proc_macro2::TokenStream {
    let ident = &f.ident;
    let info = quote! {
        #ident.to_account_info()
    };
    match c {
        ConstraintRentExempt::Skip => quote! {},
        ConstraintRentExempt::Enforce => quote! {
            if !__anchor_rent.is_exempt(#info.lamports(), #info.try_data_len()?) {
                return Err(anchor_lang::__private::ErrorCode::ConstraintRentExempt.into());
            }
        },
    }
}

fn generate_constraint_init_group(f: &Field, c: &ConstraintInitGroup) -> proc_macro2::TokenStream {
    let payer = {
        let p = &c.payer;
        quote! {
            let payer = #p.to_account_info();
        }
    };

    let seeds_with_nonce = match &c.seeds {
        None => quote! {},
        Some(c) => {
            let s = &c.seeds;
            let inner = match c.bump.as_ref() {
                // Bump target not given. Use the canonical bump.
                None => {
                    quote! {
                        [
                            #s,
                            &[
                                Pubkey::find_program_address(
                                    &[#s],
                                    program_id,
                                ).1
                            ]
                        ]
                    }
                }
                // Bump target given. Use it.
                Some(b) => quote! {
                    [#s, &[#b]]
                },
            };
            quote! {
                &#inner[..]
            }
        }
    };
    generate_pda(f, seeds_with_nonce, payer, &c.space, &c.kind)
}

fn generate_constraint_seeds(f: &Field, c: &ConstraintSeedsGroup) -> proc_macro2::TokenStream {
    let name = &f.ident;
    let s = &c.seeds;

    // If the bump is provided with init *and target*, then force it to be the
    // canonical bump.
    if c.is_init && c.bump.is_some() {
        let b = c.bump.as_ref().unwrap();
        quote! {
            let (__program_signer, __bump) = anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
                &[#s],
                program_id,
            );
            if #name.to_account_info().key != &__program_signer {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
            if __bump != #b {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
        }
    } else {
        let seeds = match c.bump.as_ref() {
            // Bump target not given. Find it.
            None => {
                quote! {
                    [
                        #s,
                        &[
                            Pubkey::find_program_address(
                                &[#s],
                                program_id,
                            ).1
                        ]
                    ]
                }
            }
            // Bump target given. Use it.
            Some(b) => {
                quote! {
                    [#s, &[#b]]
                }
            }
        };
        quote! {
            let __program_signer = Pubkey::create_program_address(
                &#seeds[..],
                program_id,
            ).map_err(|_| anchor_lang::__private::ErrorCode::ConstraintSeeds)?;
            if #name.to_account_info().key != &__program_signer {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
        }
    }
}

fn parse_ty(f: &Field) -> (proc_macro2::TokenStream, proc_macro2::TokenStream, bool) {
    match &f.ty {
        Ty::ProgramAccount(ty) => {
            let ident = &ty.account_type_path;
            (
                quote! {
                    #ident
                },
                quote! {
                    anchor_lang::ProgramAccount
                },
                false,
            )
        }
        Ty::Loader(ty) => {
            let ident = &ty.account_type_path;
            (
                quote! {
                    #ident
                },
                quote! {
                    anchor_lang::Loader
                },
                true,
            )
        }
        Ty::CpiAccount(ty) => {
            let ident = &ty.account_type_path;
            (
                quote! {
                    #ident
                },
                quote! {
                    anchor_lang::CpiAccount
                },
                false,
            )
        }
        Ty::AccountInfo => (
            quote! {
                AccountInfo
            },
            quote! {},
            false,
        ),
        _ => panic!("Invalid type for initializing a program derived address"),
    }
}

pub fn generate_pda(
    f: &Field,
    seeds_with_nonce: proc_macro2::TokenStream,
    payer: proc_macro2::TokenStream,
    space: &Option<Expr>,
    kind: &InitKind,
) -> proc_macro2::TokenStream {
    let field = &f.ident;
    let (account_ty, account_wrapper_ty, is_zero_copy) = parse_ty(f);

    let (combined_account_ty, try_from) = match f.ty {
        Ty::AccountInfo => (
            quote! {
                AccountInfo
            },
            quote! {
                #field.to_account_info()
            },
        ),
        _ => (
            quote! {
                #account_wrapper_ty<#account_ty>
            },
            quote! {
                #account_wrapper_ty::try_from_unchecked(
                    &#field.to_account_info(),
                )?
            },
        ),
    };

    match kind {
        InitKind::Token { owner, mint } => {
            let create_account = generate_create_account(
                field,
                quote! {anchor_spl::token::TokenAccount::LEN},
                quote! {token_program.to_account_info().key},
                seeds_with_nonce,
            );
            quote! {
                let #field: #combined_account_ty = {
                    // Define payer variable.
                    #payer

                    // Create the account with the system program.
                    #create_account

                    // Initialize the token account.
                    let cpi_program = token_program.to_account_info();
                    let accounts = anchor_spl::token::InitializeAccount {
                        account: #field.to_account_info(),
                        mint: #mint.to_account_info(),
                        authority: #owner.to_account_info(),
                        rent: rent.to_account_info(),
                    };
                    let cpi_ctx = CpiContext::new(cpi_program, accounts);
                    anchor_spl::token::initialize_account(cpi_ctx)?;
                    anchor_lang::CpiAccount::try_from_unchecked(
                        &#field.to_account_info(),
                    )?
                };
            }
        }
        InitKind::Mint { owner, decimals } => {
            let create_account = generate_create_account(
                field,
                quote! {anchor_spl::token::Mint::LEN},
                quote! {token_program.to_account_info().key},
                seeds_with_nonce,
            );
            quote! {
                let #field: #combined_account_ty = {
                    // Define payer variable.
                    #payer

                    // Create the account with the system program.
                    #create_account

                    // Initialize the mint account.
                    let cpi_program = token_program.to_account_info();
                    let accounts = anchor_spl::token::InitializeMint {
                        mint: #field.to_account_info(),
                        rent: rent.to_account_info(),
                    };
                    let cpi_ctx = CpiContext::new(cpi_program, accounts);
                    anchor_spl::token::initialize_mint(cpi_ctx, #decimals, &#owner.to_account_info().key, None)?;
                    anchor_lang::CpiAccount::try_from_unchecked(
                        &#field.to_account_info(),
                    )?
                };
            }
        }
        InitKind::Program { owner } => {
            let space = match space {
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

            // Owner of the account being created. If not specified,
            // default to the currently executing program.
            let owner = match owner {
                None => quote! {
                    program_id
                },
                Some(o) => quote! {
                    &#o
                },
            };
            let create_account =
                generate_create_account(field, quote! {space}, owner, seeds_with_nonce);
            quote! {
                let #field = {
                    #space
                    #payer
                    #create_account
                    let mut pa: #combined_account_ty = #try_from;
                    pa
                };
            }
        }
    }
}

// Generated code to create an account with with system program with the
// given `space` amount of data, owned by `owner`.
//
// `seeds_with_nonce` should be given for creating PDAs. Otherwise it's an
// empty stream.
pub fn generate_create_account(
    field: &Ident,
    space: proc_macro2::TokenStream,
    owner: proc_macro2::TokenStream,
    seeds_with_nonce: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        // If the account being initialized already has lamports, then
        // return them all back to the payer so that the account has
        // zero lamports when the system program's create instruction
        // is eventually called.
        let __current_lamports = #field.to_account_info().lamports();
        if __current_lamports == 0 {
            // Create the token account with right amount of lamports and space, and the correct owner.
            let lamports = __anchor_rent.minimum_balance(#space);
            anchor_lang::solana_program::program::invoke_signed(
                &anchor_lang::solana_program::system_instruction::create_account(
                    payer.to_account_info().key,
                    #field.to_account_info().key,
                    lamports,
                    #space as u64,
                    #owner,
                ),
                &[
                    payer.to_account_info(),
                    #field.to_account_info(),
                    system_program.to_account_info().clone(),
                ],
                &[#seeds_with_nonce],
            )?;
        } else {
            // Fund the account for rent exemption.
            let required_lamports = __anchor_rent
                .minimum_balance(#space)
                .max(1)
                .saturating_sub(__current_lamports);
            if required_lamports > 0 {
                anchor_lang::solana_program::program::invoke(
                    &anchor_lang::solana_program::system_instruction::transfer(
                        payer.to_account_info().key,
                        #field.to_account_info().key,
                        required_lamports,
                    ),
                    &[
                        payer.to_account_info(),
                        #field.to_account_info(),
                        system_program.to_account_info().clone(),
                    ],
                )?;
            }
            // Allocate space.
            anchor_lang::solana_program::program::invoke_signed(
                &anchor_lang::solana_program::system_instruction::allocate(
                    #field.to_account_info().key,
                    #space as u64,
                ),
                &[
                    #field.to_account_info(),
                    system_program.clone(),
                ],
                &[#seeds_with_nonce],
            )?;
            // Assign to the spl token program.
            anchor_lang::solana_program::program::invoke_signed(
                &anchor_lang::solana_program::system_instruction::assign(
                    #field.to_account_info().key,
                    #owner,
                ),
                &[
                    #field.to_account_info(),
                    system_program.to_account_info(),
                ],
                &[#seeds_with_nonce],
            )?;
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
            return Err(anchor_lang::__private::ErrorCode::ConstraintExecutable.into());
        }
    }
}

pub fn generate_constraint_state(f: &Field, c: &ConstraintState) -> proc_macro2::TokenStream {
    let program_target = c.program_target.clone();
    let ident = &f.ident;
    let account_ty = match &f.ty {
        Ty::CpiState(ty) => &ty.account_type_path,
        _ => panic!("Invalid state constraint"),
    };
    quote! {
        // Checks the given state account is the canonical state account for
        // the target program.
        if #ident.to_account_info().key != &anchor_lang::CpiState::<#account_ty>::address(#program_target.to_account_info().key) {
            return Err(anchor_lang::__private::ErrorCode::ConstraintState.into());
        }
        if #ident.to_account_info().owner != #program_target.to_account_info().key {
            return Err(anchor_lang::__private::ErrorCode::ConstraintState.into());
        }
    }
}
