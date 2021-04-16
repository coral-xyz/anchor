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
    let is_zero_copy = match args.into_iter().next() {
        None => false,
        Some(tt) => match tt {
            proc_macro::TokenTree::Literal(_) => false,
            _ => namespace == "zero_copy",
        },
    };

    let account_strct = parse_macro_input!(input as syn::ItemStruct);
    let account_name = &account_strct.ident;

    let discriminator: proc_macro2::TokenStream = {
        // Namespace the discriminator to prevent collisions.
        let discriminator_preimage = {
            // For now, zero copy accounts can't be namespaced.
            if is_zero_copy || namespace.is_empty() {
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

    proc_macro::TokenStream::from({
        if is_zero_copy {
            quote! {
                #[repr(packed)]
                #[derive(anchor_lang::__private::ZeroCopyAccessor, Copy, Clone)]
                #account_strct

                unsafe impl anchor_lang::__private::bytemuck::Zeroable for #account_name {}
                unsafe impl anchor_lang::__private::bytemuck::Pod for #account_name {}
                unsafe impl anchor_lang::__private::safe_transmute::trivial::TriviallyTransmutable for #account_name {}

                impl anchor_lang::AccountDeserializeZeroCopy for #account_name {
                    fn try_deserialize<'info>(buf: &'info mut [u8]) -> std::result::Result<&'info mut Self, ProgramError> {
                        if buf.len() < #discriminator.len() {
                            return Err(ProgramError::AccountDataTooSmall);
                        }
                        let given_disc = &buf[..8];
                        if &#discriminator != given_disc {
                            return Err(ProgramError::InvalidInstructionData);
                        }
                        Self::try_deserialize_unchecked(buf)
                    }

                    fn try_deserialize_unchecked<'info>(buf: &'info mut [u8]) -> std::result::Result<&'info mut Self, ProgramError> {
                        let mut data: &mut [u8] = &mut buf[8..];
                        Ok(anchor_lang::__private::bytemuck::from_bytes_mut(data))
                    }
                }

                impl anchor_lang::Discriminator for #account_name {
                    fn discriminator() -> [u8; 8] {
                        #discriminator
                    }
                }

                impl anchor_lang::ZeroCopy for #account_name {}
            }
        } else {
            quote! {
                #[derive(AnchorSerialize, AnchorDeserialize, Clone)]
                #account_strct

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
            }
        }
    })
}

/// Extends the `#[account]` attribute to allow one to create associated token
/// accounts. This includes a `Default` implementation, which means all fields
/// in an `#[associated]` struct must implement `Default` and an
/// `anchor_lang::Bump` trait implementation, which allows the account to be
/// used as a program derived address.
#[proc_macro_attribute]
pub fn associated(
    args: proc_macro::TokenStream,
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

    let args: proc_macro2::TokenStream = args.into();
    proc_macro::TokenStream::from(quote! {
        #[anchor_lang::account(#args)]
        #[derive(Default)]
        #account_strct

        impl anchor_lang::Bump for #account_name {
            fn seed(&self) -> u8 {
                self.__nonce
            }
        }
    })
}

#[proc_macro_derive(ZeroCopyAccessor, attributes(accessor))]
pub fn derive_zero_copy_accessor(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let account_strct = parse_macro_input!(item as syn::ItemStruct);
    let account_name = &account_strct.ident;

    let fields = match &account_strct.fields {
        syn::Fields::Named(n) => n,
        _ => panic!("Fields must be named"),
    };
    let methods: Vec<proc_macro2::TokenStream> = fields
        .named
        .iter()
        .filter_map(|field: &syn::Field| {
            field
                .attrs
                .iter()
                .filter(|attr| {
                    let name = anchor_syn::parser::tts_to_string(&attr.path);
                    if name != "accessor" {
                        return false;
                    }
                    return true;
                })
                .next()
                .map(|attr| {
                    let mut tts = attr.tokens.clone().into_iter();
                    let g_stream = match tts.next().expect("Must have a token group") {
                        proc_macro2::TokenTree::Group(g) => g.stream(),
                        _ => panic!("Invalid syntax"),
                    };
                    let accessor_ty = match g_stream.into_iter().next() {
                        Some(token) => token,
                        _ => panic!("Missing accessor type"),
                    };

                    let field_name = field.ident.as_ref().unwrap();

                    let get_field: proc_macro2::TokenStream =
                        format!("get_{}", field_name.to_string()).parse().unwrap();
                    let set_field: proc_macro2::TokenStream =
                        format!("set_{}", field_name.to_string()).parse().unwrap();

                    quote! {
                        pub fn #get_field(&self) -> #accessor_ty {
                            anchor_lang::ZeroCopyAccessor::get(&self.#field_name)
                        }
                        pub fn #set_field(&mut self, input: &#accessor_ty) {
                            self.#field_name = anchor_lang::ZeroCopyAccessor::set(input);
                        }
                    }
                })
        })
        .collect();
    proc_macro::TokenStream::from(quote! {
        impl #account_name {
            #(#methods)*
        }
    })
}

/// Marks a type so that it can be used as a field inside a
/// `#[account(zero_copy)].
///
/// This is just a convenient alias for
///
/// ```
/// #[derive(Copy, Clone)]
/// #[repr(packed)]
/// struct MyStruct {...}
/// ```
#[proc_macro_attribute]
pub fn zero_copy(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let account_strct = parse_macro_input!(item as syn::ItemStruct);

    proc_macro::TokenStream::from(quote! {
            #[derive(Copy, Clone)]
            #[repr(packed)]
            #account_strct
    })
}
