use std::{
    collections::BTreeMap,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::anyhow;
use syn::{Ident, ImplItem, ImplItemConst, Type, TypePath};

/// Crate parse context
///
/// Keeps track of modules defined within a crate.
pub struct CrateContext {
    modules: BTreeMap<String, ParsedModule>,
}

impl CrateContext {
    pub fn consts(&self) -> impl Iterator<Item = &syn::ItemConst> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.consts())
    }

    pub fn impls(&self) -> impl Iterator<Item = &syn::ItemImpl> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.impls())
    }

    pub fn impl_consts(&self) -> impl Iterator<Item = (&Ident, &syn::ImplItemConst)> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.impl_consts())
    }

    pub fn structs(&self) -> impl Iterator<Item = &syn::ItemStruct> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.structs())
    }

    pub fn enums(&self) -> impl Iterator<Item = &syn::ItemEnum> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.enums())
    }

    pub fn modules(&self) -> impl Iterator<Item = ModuleContext> {
        self.modules.values().map(|detail| ModuleContext { detail })
    }

    pub fn uses(&self) -> impl Iterator<Item = &syn::ItemUse> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.uses())
    }

    pub fn root_module(&self) -> ModuleContext {
        ModuleContext {
            detail: self.modules.get("crate").unwrap(),
        }
    }

    pub fn parse(
        root: impl AsRef<Path>,
        features: &Option<Vec<String>>,
    ) -> Result<Self, anyhow::Error> {
        Ok(CrateContext {
            modules: ParsedModule::parse_recursive(root.as_ref(), features)?,
        })
    }

    // Perform Anchor safety checks on the parsed create
    pub fn safety_checks(&self) -> Result<(), anyhow::Error> {
        // Check all structs for unsafe field types, i.e. AccountInfo and UncheckedAccount.
        for (_, ctx) in self.modules.iter() {
            for unsafe_field in ctx.unsafe_struct_fields() {
                // Check if unsafe field type has been documented with a /// SAFETY: doc string.
                let is_documented = unsafe_field.attrs.iter().any(|attr| {
                    attr.tokens.clone().into_iter().any(|token| match token {
                        // Check for doc comments containing CHECK
                        proc_macro2::TokenTree::Literal(s) => s.to_string().contains("CHECK"),
                        _ => false,
                    })
                });
                if !is_documented {
                    let ident = unsafe_field.ident.as_ref().unwrap();
                    let span = ident.span();
                    // Error if undocumented.
                    return Err(anyhow!(
                        r#"
        {}:{}
        Struct field "{}" is unsafe, but is not documented.
        Please add a `/// CHECK:` doc comment explaining why no checks through types are necessary.
        See https://www.anchor-lang.com/docs/the-accounts-struct#safety-checks for more information.
                    "#,
                        span.start().line,
                        span.start().column,
                        ident.to_string()
                    ));
                };
            }
        }
        Ok(())
    }
}

/// Module parse context
///
/// Keeps track of items defined within a module.
#[derive(Copy, Clone)]
pub struct ModuleContext<'krate> {
    detail: &'krate ParsedModule,
}

impl<'krate> ModuleContext<'krate> {
    pub fn items(&self) -> impl Iterator<Item = &syn::Item> {
        self.detail.items.iter()
    }
}

struct ParsedModule {
    name: String,
    items: Vec<syn::Item>,
}

impl ParsedModule {
    fn parse_recursive(
        root: &Path,
        features: &Option<Vec<String>>,
    ) -> Result<BTreeMap<String, ParsedModule>, anyhow::Error> {
        let mut modules = BTreeMap::new();

        let mut args = vec![
            "+nightly".to_owned(),
            "rustc".to_owned(),
            "--profile=check".to_owned(),
        ];
        if let Some(features) = features {
            args.push("--features".to_owned());
            args.push(features.join(","));
        }
        args.extend(vec!["--".to_owned(), "-Zunpretty=expanded".to_owned()]);

        let root_content = String::from_utf8(
            Command::new("cargo")
                .args(args)
                .current_dir(root)
                .stderr(Stdio::inherit())
                .output()?
                .stdout,
        )?;

        let root_file = syn::parse_file(&root_content)?;
        let root_mod = Self::new("crate".to_owned(), root_file.items);

        struct UnparsedModule {
            name: String,
            item: syn::ItemMod,
        }

        let mut unparsed = root_mod
            .submodules()
            .map(|item| UnparsedModule {
                name: item.ident.to_string(),
                item: item.clone(),
            })
            .collect::<Vec<_>>();

        while let Some(to_parse) = unparsed.pop() {
            let name = to_parse.name;
            let module = Self::from_item_mod(to_parse.item)?;

            unparsed.extend(module.submodules().map(|item| UnparsedModule {
                item: item.clone(),
                name: item.ident.to_string(),
            }));
            modules.insert(name.clone(), module);
        }

        modules.insert(root_mod.name.clone(), root_mod);

        Ok(modules)
    }

    fn from_item_mod(item: syn::ItemMod) -> syn::parse::Result<Self> {
        Ok(match item.content {
            Some((_, items)) => {
                // The module content is within the parent file being parsed
                Self::new(item.ident.to_string(), items)
            }
            None => {
                // NOTE(vadorovsky): Upstream Anchor assumes that in this case
                // the module is referencing some other file. With our current
                // approach of expanding macros of the whole crate at once,
                // that assumption is gone.
                // But we might still (unlikely) encounter that case if there
                // is an empty module. We are going to just return an empty
                // module in that case.
                Self::new(item.ident.to_string(), Vec::new())
            }
        })
    }

    fn new(name: String, items: Vec<syn::Item>) -> Self {
        Self { name, items }
    }

    fn submodules(&self) -> impl Iterator<Item = &syn::ItemMod> {
        let mut res = Vec::new();

        fn submodules_recursive(items: &[syn::Item]) -> Vec<&syn::ItemMod> {
            let mut res = Vec::new();
            for item in items.iter() {
                if let syn::Item::Mod(item) = item {
                    let ident = item.ident.to_string();
                    if ident.starts_with("__") || ident == "instruction" {
                        continue;
                    }
                    res.push(item);
                    if let Some((_, items)) = &item.content {
                        res.extend(submodules_recursive(items));
                    }
                }
            }
            res
        }

        res.extend(submodules_recursive(&self.items));
        res.into_iter()
    }

    fn structs(&self) -> impl Iterator<Item = &syn::ItemStruct> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Struct(item) => Some(item),
            _ => None,
        })
    }

    fn unsafe_struct_fields(&self) -> impl Iterator<Item = &syn::Field> {
        let accounts_filter = |item_struct: &&syn::ItemStruct| {
            item_struct.attrs.iter().any(|attr| {
                match attr.parse_meta() {
                    Ok(syn::Meta::List(syn::MetaList{path, nested, ..})) => {
                        path.is_ident("derive") && nested.iter().any(|nested| {
                            matches!(nested, syn::NestedMeta::Meta(syn::Meta::Path(path)) if path.is_ident("Accounts"))
                        })
                    }
                    _ => false
                }
            })
        };

        self.structs()
            .filter(accounts_filter)
            .flat_map(|s| &s.fields)
            .filter(|f| match &f.ty {
                syn::Type::Path(syn::TypePath {
                    path: syn::Path { segments, .. },
                    ..
                }) => {
                    segments.len() == 1 && segments[0].ident == "UncheckedAccount"
                        || segments[0].ident == "AccountInfo"
                }
                _ => false,
            })
    }

    fn enums(&self) -> impl Iterator<Item = &syn::ItemEnum> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Enum(item) => Some(item),
            _ => None,
        })
    }

    fn consts(&self) -> impl Iterator<Item = &syn::ItemConst> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Const(item) => Some(item),
            _ => None,
        })
    }

    fn impls(&self) -> impl Iterator<Item = &syn::ItemImpl> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Impl(item) => Some(item),
            _ => None,
        })
    }

    fn impl_consts(&self) -> impl Iterator<Item = (&Ident, &ImplItemConst)> {
        self.items
            .iter()
            .filter_map(|i| match i {
                syn::Item::Impl(syn::ItemImpl {
                    self_ty: ty, items, ..
                }) => {
                    if let Type::Path(TypePath {
                        qself: None,
                        path: p,
                    }) = ty.as_ref()
                    {
                        if let Some(ident) = p.get_ident() {
                            let mut to_return = Vec::new();
                            items.iter().for_each(|item| {
                                if let ImplItem::Const(item) = item {
                                    to_return.push((ident, item));
                                }
                            });
                            Some(to_return)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .flatten()
    }

    fn uses(&self) -> impl Iterator<Item = &syn::ItemUse> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Use(item) => Some(item),
            _ => None,
        })
    }
}
