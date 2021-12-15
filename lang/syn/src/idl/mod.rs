use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use syn::{Expr, GenericArgument, Lit, PathArguments, Type};

pub mod file;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Idl {
    pub version: String,
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub constants: Vec<IdlConst>,
    pub instructions: Vec<IdlInstruction>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub state: Option<IdlState>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub accounts: Vec<IdlTypeDefinition>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub types: Vec<IdlTypeDefinition>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub events: Option<Vec<IdlEvent>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub errors: Option<Vec<IdlErrorCode>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlConst {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: IdlType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlState {
    #[serde(rename = "struct")]
    pub strct: IdlTypeDefinition,
    pub methods: Vec<IdlInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlInstruction {
    pub name: String,
    pub accounts: Vec<IdlAccountItem>,
    pub args: Vec<IdlField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdlAccounts {
    pub name: String,
    pub accounts: Vec<IdlAccountItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum IdlAccountItem {
    IdlAccount(IdlAccount),
    IdlAccounts(IdlAccounts),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdlAccount {
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlField {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: IdlType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlEvent {
    pub name: String,
    pub fields: Vec<IdlEventField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlEventField {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: IdlType,
    pub index: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlTypeDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: IdlTypeDefinitionTy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum IdlTypeDefinitionTy {
    Struct { fields: Vec<IdlField> },
    Enum { variants: Vec<IdlEnumVariant> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlEnumVariant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fields: Option<EnumFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EnumFields {
    Named(Vec<IdlField>),
    Tuple(Vec<IdlType>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum IdlType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    Bytes,
    String,
    PublicKey,
    Defined(String),
    Option(Box<IdlType>),
    Vec(Box<IdlType>),
    Array(Box<IdlType>, usize),
    Map(Box<IdlType>, Box<IdlType>),
}

impl std::str::FromStr for IdlType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn syn_type_to_idl_type(syn_type: &Type) -> Result<IdlType, anyhow::Error> {
            match syn_type {
                Type::Path(type_path) => {
                    let type_word = type_path.path.segments[0].ident.to_string();
                    let type_args: Vec<Type> = {
                        match &type_path.path.segments[0].arguments {
                            PathArguments::AngleBracketed(x) => x
                                .args
                                .iter()
                                .map(|arg| match arg {
                                    GenericArgument::Type(ty) => Ok(ty.clone()),
                                    _ => Err(()),
                                })
                                .collect::<Result<Vec<Type>, _>>(),
                            _ => Ok(Vec::new()),
                        }
                        .expect("Invalid option")
                    };
                    let r = match type_word.as_str() {
                        "bool" => IdlType::Bool,
                        "u8" => IdlType::U8,
                        "i8" => IdlType::I8,
                        "u16" => IdlType::U16,
                        "i16" => IdlType::I16,
                        "u32" => IdlType::U32,
                        "i32" => IdlType::I32,
                        "u64" => IdlType::U64,
                        "i64" => IdlType::I64,
                        "u128" => IdlType::U128,
                        "i128" => IdlType::I128,
                        "Vec<u8>" => IdlType::Bytes,
                        "String" => IdlType::String,
                        "Pubkey" => IdlType::PublicKey,
                        "Option" => {
                            if type_args.len() != 1 {
                                return Err(anyhow::anyhow!("Invalid option"));
                            }
                            let inner_ty = syn_type_to_idl_type(&type_args[0])?;
                            IdlType::Option(Box::new(inner_ty))
                        }
                        "Vec" | "VecDeque" | "LinkedList" => {
                            if type_args.len() != 1 {
                                return Err(anyhow::anyhow!("Invalid option"));
                            }
                            let inner_ty = syn_type_to_idl_type(&type_args[0])?;
                            IdlType::Vec(Box::new(inner_ty))
                        }
                        "BTreeMap" => {
                            if type_args.len() != 2 {
                                return Err(anyhow::anyhow!("Invalid option"));
                            }
                            let key_ty = syn_type_to_idl_type(&type_args[0])?;
                            let value_ty = syn_type_to_idl_type(&type_args[1])?;
                            IdlType::Map(Box::new(key_ty), Box::new(value_ty))
                        }
                        _ => IdlType::Defined(type_word),
                    };
                    Ok(r)
                }
                Type::Array(type_array) => {
                    let inner_ty = syn_type_to_idl_type(&(*type_array.elem))?;
                    let size: usize = {
                        match &type_array.len {
                            Expr::Lit(x) => match &x.lit {
                                Lit::Int(x) => x.base10_parse::<usize>().map_err(|_| ()),
                                _ => Err(()),
                            },
                            _ => Err(()),
                        }
                    }
                    .expect("Invalid option");
                    Ok(IdlType::Array(Box::new(inner_ty), size))
                }
                _ => Err(anyhow::anyhow!("Invalid option")),
            }
        }

        let ty = syn::parse_str::<Type>(s)?;
        syn_type_to_idl_type(&ty)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlErrorCode {
    pub code: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub msg: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::idl::IdlType;
    use std::str::FromStr;

    #[test]
    fn multidimensional_array() {
        assert_eq!(
            IdlType::from_str("[[u8;16];32]").unwrap(),
            IdlType::Array(Box::new(IdlType::Array(Box::new(IdlType::U8), 16)), 32)
        );
    }

    #[test]
    fn array() {
        assert_eq!(
            IdlType::from_str("[Pubkey;16]").unwrap(),
            IdlType::Array(Box::new(IdlType::PublicKey), 16)
        );
    }

    #[test]
    fn array_with_underscored_length() {
        assert_eq!(
            IdlType::from_str("[u8;50_000]").unwrap(),
            IdlType::Array(Box::new(IdlType::U8), 50000)
        );
    }

    #[test]
    fn option() {
        assert_eq!(
            IdlType::from_str("Option<bool>").unwrap(),
            IdlType::Option(Box::new(IdlType::Bool))
        )
    }

    #[test]
    fn vector() {
        assert_eq!(
            IdlType::from_str("Vec<bool>").unwrap(),
            IdlType::Vec(Box::new(IdlType::Bool))
        )
    }

    #[test]
    fn map() {
        assert_eq!(
            IdlType::from_str("BTreeMap<u32, Pubkey>").unwrap(),
            IdlType::Map(Box::new(IdlType::U32), Box::new(IdlType::PublicKey))
        )
    }

    #[test]
    fn map_with_multiple_angle_brackets() {
        assert_eq!(
            IdlType::from_str("BTreeMap<Vec<u8>, Pubkey>").unwrap(),
            IdlType::Map(
                Box::new(IdlType::Vec(Box::new(IdlType::U8))),
                Box::new(IdlType::PublicKey)
            )
        )
    }
}
