extern crate proc_macro;

use quote::quote;
use syn::parse_macro_input;

/// A data structure representing a Solana account, implementing various traits:
///
/// - [`AccountSerialize`](./trait.AccountSerialize.html)
/// - [`AccountDeserialize`](./trait.AccountDeserialize.html)
/// - [`AnchorSerialize`](./trait.AnchorSerialize.html)
/// - [`AnchorDeserialize`](./trait.AnchorDeserialize.html)
///
/// When implementing account serialization traits the first 8 bytes are
/// reserved for a unique account discriminator, self described by the first 8
/// bytes of the SHA256 of the account's Rust ident.
///
/// As a result, any calls to `AccountDeserialize`'s `try_deserialize` will
/// check this discriminator. If it doesn't match, an invalid account was given,
/// and the account deserialization will exit with an error.
#[proc_macro_attribute]
pub fn account(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let namespace = args.to_string().replace("\"", "");

    let account_strct = parse_macro_input!(input as syn::ItemStruct);
    let account_name = &account_strct.ident;

    let discriminator: proc_macro2::TokenStream = {
        // Namespace the discriminator to prevent collisions.
        let discriminator_preimage = {
            if namespace.is_empty() {
                format!("account:{}", account_name.to_string())
            } else {
                format!("{}:{}", namespace, account_name.to_string())
            }
        };

        let mut discriminator = [0u8; 8];
        discriminator.copy_from_slice(
            &anchor_syn::hash::hash(discriminator_preimage.as_bytes()).to_bytes()[..8],
        );
        format!("{:?}", discriminator).parse().unwrap()
    };

    let coder = quote! {
        impl anchor_lang::AccountSerialize for #account_name {
            fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> std::result::Result<(), ProgramError> {
                writer.write_all(&#discriminator).map_err(|_| ProgramError::InvalidAccountData)?;
                AnchorSerialize::serialize(
                    self,
                    writer
                )
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                Ok(())
            }
        }

        impl anchor_lang::AccountDeserialize for #account_name {
            fn try_deserialize(buf: &mut &[u8]) -> std::result::Result<Self, ProgramError> {
                 if buf.len() < #discriminator.len() {
                    return Err(ProgramError::AccountDataTooSmall);
                }
                let given_disc = &buf[..8];
                if &#discriminator != given_disc {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Self::try_deserialize_unchecked(buf)
            }

            fn try_deserialize_unchecked(buf: &mut &[u8]) -> std::result::Result<Self, ProgramError> {
                let mut data: &[u8] = &buf[8..];
                AnchorDeserialize::deserialize(&mut data)
                    .map_err(|_| ProgramError::InvalidAccountData)
            }
        }

        impl anchor_lang::Discriminator for #account_name {
            fn discriminator() -> [u8; 8] {
                #discriminator
            }
        }
    };

    proc_macro::TokenStream::from(quote! {
        #[derive(AnchorSerialize, AnchorDeserialize, Clone)]
        #account_strct

        #coder
    })
}

/// Extends the `#[account]` attribute to allow one to create associated token
/// accounts. This includes a `Default` implementation, which means all fields
/// in an `#[associated]` struct must implement `Default` and an
/// `anchor_lang::Bump` trait implementation, which allows the account to be
/// used as a program derived address.
#[proc_macro_attribute]
pub fn associated(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut account_strct = parse_macro_input!(input as syn::ItemStruct);
    let account_name = &account_strct.ident;

    // Add a `__nonce: u8` field to the struct to hold the bump seed for
    // the program dervied address.
    match &mut account_strct.fields {
        syn::Fields::Named(fields) => {
            let mut segments = syn::punctuated::Punctuated::new();
            segments.push(syn::PathSegment {
                ident: syn::Ident::new("u8", proc_macro2::Span::call_site()),
                arguments: syn::PathArguments::None,
            });
            fields.named.push(syn::Field {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                ident: Some(syn::Ident::new("__nonce", proc_macro2::Span::call_site())),
                colon_token: Some(syn::token::Colon {
                    spans: [proc_macro2::Span::call_site()],
                }),
                ty: syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments,
                    },
                }),
            });
        }
        _ => panic!("Fields must be named"),
    }

    proc_macro::TokenStream::from(quote! {
        #[anchor_lang::account]
        #[derive(Default)]
        #account_strct

        impl anchor_lang::Bump for #account_name {
            fn seed(&self) -> u8 {
                self.__nonce
            }
        }
    })
}
