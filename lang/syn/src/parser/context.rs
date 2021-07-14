use std::collections::HashMap;
use std::path::{Path, PathBuf};

use syn::parse::{Error as ParseError, Result as ParseResult};

/// Crate parse context
///
/// Keeps track of modules defined within a crate.
pub struct CrateContext {
    modules: HashMap<String, ParsedModule>,
}

impl CrateContext {
    pub fn structs(&self) -> impl Iterator<Item = &syn::ItemStruct> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.structs())
    }

    pub fn enums(&self) -> impl Iterator<Item = &syn::ItemEnum> {
        self.modules.iter().flat_map(|(_, ctx)| ctx.enums())
    }

    pub fn modules(&self) -> impl Iterator<Item = ModuleContext> {
        self.modules
            .iter()
            .map(move |(_, detail)| ModuleContext { detail })
    }

    pub fn root_module(&self) -> ModuleContext {
        ModuleContext {
            detail: self.modules.get("crate").unwrap(),
        }
    }

    pub fn parse(root: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        Ok(CrateContext {
            modules: ParsedModule::parse_recursive(root.as_ref())?,
        })
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
    file: PathBuf,
    path: String,
    items: Vec<syn::Item>,
}

impl ParsedModule {
    fn parse_recursive(root: &Path) -> Result<HashMap<String, ParsedModule>, anyhow::Error> {
        let mut modules = HashMap::new();

        let root_content = std::fs::read_to_string(root)?;
        let root_file = syn::parse_file(&root_content)?;
        let root_mod = Self::new(
            String::new(),
            root.to_owned(),
            "crate".to_owned(),
            root_file.items,
        );

        struct UnparsedModule {
            file: PathBuf,
            path: String,
            name: String,
            item: syn::ItemMod,
        }

        let mut unparsed = root_mod
            .submodules()
            .map(|item| UnparsedModule {
                file: root_mod.file.clone(),
                path: root_mod.path.clone(),
                name: item.ident.to_string(),
                item: item.clone(),
            })
            .collect::<Vec<_>>();

        while let Some(to_parse) = unparsed.pop() {
            let path = format!("{}::{}", to_parse.path, to_parse.name);
            let name = to_parse.name;
            let module = Self::from_item_mod(&to_parse.file, &path, to_parse.item)?;

            unparsed.extend(module.submodules().map(|item| UnparsedModule {
                item: item.clone(),
                file: module.file.clone(),
                path: module.path.clone(),
                name: name.clone(),
            }));
            modules.insert(name.clone(), module);
        }

        modules.insert(root_mod.name.clone(), root_mod);

        Ok(modules)
    }

    fn from_item_mod(
        parent_file: &Path,
        parent_path: &str,
        item: syn::ItemMod,
    ) -> ParseResult<Self> {
        let path = format!("{}::{}", parent_path, item.ident);

        Ok(match item.content {
            Some((_, items)) => {
                // The module content is within the parent file being parsed
                Self::new(path, parent_file.to_owned(), item.ident.to_string(), items)
            }
            None => {
                // The module is referencing some other file, so we need to load that
                // to parse the items it has.
                let parent_dir = parent_file.parent().unwrap();
                let parent_filename = parent_file.file_stem().unwrap().to_str().unwrap();
                let parent_mod_dir = parent_dir.join(parent_filename);

                let possible_file_paths = vec![
                    parent_dir.join(format!("{}.rs", item.ident)),
                    parent_dir.join(format!("{}/mod.rs", item.ident)),
                    parent_mod_dir.join(format!("{}.rs", item.ident)),
                    parent_mod_dir.join(format!("{}/mod.rs", item.ident)),
                ];

                let mod_file_path = possible_file_paths
                    .into_iter()
                    .find(|p| p.exists())
                    .ok_or_else(|| ParseError::new_spanned(&item, "could not find file"))?;
                let mod_file_content = std::fs::read_to_string(&mod_file_path)
                    .map_err(|_| ParseError::new_spanned(&item, "could not read file"))?;
                let mod_file = syn::parse_file(&mod_file_content)?;

                Self::new(path, mod_file_path, item.ident.to_string(), mod_file.items)
            }
        })
    }

    fn new(path: String, file: PathBuf, name: String, items: Vec<syn::Item>) -> Self {
        Self {
            name,
            file,
            path,
            items,
        }
    }

    fn submodules(&self) -> impl Iterator<Item = &syn::ItemMod> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Mod(item) => Some(item),
            _ => None,
        })
    }

    fn structs(&self) -> impl Iterator<Item = &syn::ItemStruct> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Struct(item) => Some(item),
            _ => None,
        })
    }

    fn enums(&self) -> impl Iterator<Item = &syn::ItemEnum> {
        self.items.iter().filter_map(|i| match i {
            syn::Item::Enum(item) => Some(item),
            _ => None,
        })
    }
}
