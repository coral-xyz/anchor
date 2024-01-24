#[cfg(feature = "interface-instructions")]
use syn::{Lit, Meta, NestedMeta};

#[cfg(not(feature = "interface-instructions"))]
pub fn parse(_attrs: &[syn::Attribute]) -> Option<([u8; 8], String)> {
    None
}

#[derive(Clone)]
pub struct NameOverrides {
    pub namespace: Option<String>,
    pub name: Option<String>,
}

#[cfg(feature = "interface-instructions")]
pub fn parse(attrs: &[syn::Attribute]) -> NameOverrides {
    let interfaces: Vec<NameOverrides> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path.is_ident("ix") {
                let mut namespace: Option<String> = None;
                let mut ix_name: Option<String> = None;

                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested_meta in meta_list.nested.iter() {
                        if let NestedMeta::Meta(Meta::NameValue(namevalue)) = nested_meta {
                            if namevalue.path.segments[0].ident.to_string() == "namespace" {
                                if let Lit::Str(namespace_override) = &namevalue.lit {
                                    namespace = Some(namespace_override.value().clone());
                                }
                            }

                            if namevalue.path.segments[0].ident.to_string() == "name" {
                                if let Lit::Str(name_override) = &namevalue.lit {
                                    ix_name = Some(name_override.value().clone());
                                }
                            }
                        }
                    }
                    return Some(NameOverrides {
                        namespace,
                        name: ix_name,
                    });
                }
                panic!(
                    "Failed to parse interface instruction:\n{:?} {:?} {:?}",
                    quote::quote!(#attr),
                    ix_name,
                    namespace
                );
            }
            None
        })
        .collect();
    if interfaces.len() > 1 {
        panic!("An instruction can only implement one interface instruction");
    } else if interfaces.is_empty() {
        NameOverrides {
            namespace: None,
            name: None,
        }
    } else {
        interfaces[0].clone()
    }
}
