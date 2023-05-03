use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;

pub mod file;
pub mod pda;
pub mod relations;

#[derive(Debug, Error)]
pub enum IdlError {
    // This error should be handled by inside IDL parser and **not** returned
    // to the end user.
    #[error("Type to skip")]
    TypeToSkip,

    #[error("Could not parse ; delimiter from array type: {0}")]
    ArrayDelimiter(String),
    #[error("Could not parse length from array type: {0}")]
    ArrayLength(String),
    #[error("Invalid option: {0}")]
    Option(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Idl {
    pub version: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub docs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub constants: Vec<IdlConst>,
    pub instructions: Vec<IdlInstruction>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<Vec<String>>,
    pub accounts: Vec<IdlAccountItem>,
    pub args: Vec<IdlField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<IdlType>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_optional: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pda: Option<IdlPda>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub relations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdlPda {
    pub seeds: Vec<IdlSeed>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub program_id: Option<IdlSeed>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum IdlSeed {
    Const(IdlSeedConst),
    Arg(IdlSeedArg),
    Account(IdlSeedAccount),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdlSeedAccount {
    #[serde(rename = "type")]
    pub ty: IdlType,
    // account_ty points to the entry in the "accounts" section.
    // Some only if the `Account<T>` type is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdlSeedArg {
    #[serde(rename = "type")]
    pub ty: IdlType,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdlSeedConst {
    #[serde(rename = "type")]
    pub ty: IdlType,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlField {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<Vec<String>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<Vec<String>>,
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
    F32,
    U64,
    I64,
    F64,
    U128,
    I128,
    U256,
    I256,
    Bytes,
    String,
    PublicKey,
    Defined(String),
    Option(Box<IdlType>),
    Vec(Box<IdlType>),
    Array(Box<IdlType>, usize),
}

impl IdlType {
    fn from_str(s: &str) -> Result<Option<Self>, IdlError> {
        let mut s = s.to_string();
        fn array_from_str(inner: &str) -> Result<Option<IdlType>, IdlError> {
            match inner.strip_suffix(']') {
                None => {
                    let (raw_type, raw_length) = inner
                        .rsplit_once(';')
                        .ok_or(IdlError::ArrayDelimiter(inner.to_owned()))?;
                    match IdlType::from_str(raw_type)? {
                        Some(ty) => {
                            let len = raw_length
                                .replace('_', "")
                                .parse::<usize>()
                                .map_err(|_| IdlError::ArrayLength(inner.to_owned()))?;
                            Ok(Some(IdlType::Array(Box::new(ty), len)))
                        }
                        None => Ok(None),
                    }
                }
                Some(nested_inner) => array_from_str(&nested_inner[1..]),
            }
        }
        s.retain(|c| !c.is_whitespace());

        Ok(match s.as_str() {
            "bool" => Some(IdlType::Bool),
            "u8" => Some(IdlType::U8),
            "i8" => Some(IdlType::I8),
            "u16" => Some(IdlType::U16),
            "i16" => Some(IdlType::I16),
            "u32" => Some(IdlType::U32),
            "i32" => Some(IdlType::I32),
            "f32" => Some(IdlType::F32),
            "u64" => Some(IdlType::U64),
            "i64" => Some(IdlType::I64),
            "f64" => Some(IdlType::F64),
            "u128" => Some(IdlType::U128),
            "i128" => Some(IdlType::I128),
            "u256" => Some(IdlType::U256),
            "i256" => Some(IdlType::I256),
            "Vec<u8>" => Some(IdlType::Bytes),
            "String" | "&str" | "&'staticstr" => Some(IdlType::String),
            "Pubkey" => Some(IdlType::PublicKey),
            _ => {
                // Skip marker types, which have no relevance to TypeScript.
                if s.starts_with("PhantomData") || s.starts_with("PhantomPinned") {
                    return Ok(None);
                }

                // Handle options, vectors, array or defined types.
                if let Some(inner) = s.strip_prefix("Option<") {
                    Self::from_str(
                        inner
                            .strip_suffix('>')
                            .ok_or_else(|| IdlError::Option(s.clone()))?,
                    )?
                    .map(|inner_ty| IdlType::Option(Box::new(inner_ty)))
                } else if let Some(inner) = s.strip_prefix("Vec<") {
                    Self::from_str(
                        inner
                            .strip_suffix('>')
                            .ok_or_else(|| IdlError::Option(s.clone()))?,
                    )?
                    .map(|inner_ty| IdlType::Vec(Box::new(inner_ty)))
                } else if s.starts_with('[') {
                    array_from_str(&s)?
                } else {
                    // Make sure that we remove generics from the type name.
                    // For example, that we convert `MyStruct<T>` to `MyStruct`.
                    let s = s.split('<').next().unwrap().to_string();
                    Some(IdlType::Defined(s))
                }
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlErrorCode {
    pub code: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub msg: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::idl::IdlType;

    #[test]
    fn multidimensional_array() {
        assert_eq!(
            IdlType::from_str("[[u8;16];32]").unwrap().unwrap(),
            IdlType::Array(Box::new(IdlType::Array(Box::new(IdlType::U8), 16)), 32)
        );
    }

    #[test]
    fn array() {
        assert_eq!(
            IdlType::from_str("[Pubkey;16]").unwrap().unwrap(),
            IdlType::Array(Box::new(IdlType::PublicKey), 16)
        );
    }

    #[test]
    fn array_with_underscored_length() {
        assert_eq!(
            IdlType::from_str("[u8;50_000]").unwrap().unwrap(),
            IdlType::Array(Box::new(IdlType::U8), 50000)
        );
    }

    #[test]
    fn option() {
        assert_eq!(
            IdlType::from_str("Option<bool>").unwrap().unwrap(),
            IdlType::Option(Box::new(IdlType::Bool))
        )
    }

    #[test]
    fn vector() {
        assert_eq!(
            IdlType::from_str("Vec<bool>").unwrap().unwrap(),
            IdlType::Vec(Box::new(IdlType::Bool))
        )
    }

    #[test]
    fn defined() {
        assert_eq!(
            IdlType::from_str("MyStruct").unwrap().unwrap(),
            IdlType::Defined("MyStruct".to_string())
        );
    }

    #[test]
    fn defined_with_generics() {
        assert_eq!(
            IdlType::from_str("MyStruct<T>").unwrap().unwrap(),
            IdlType::Defined("MyStruct".to_string())
        );
        assert_eq!(
            IdlType::from_str("MyStruct<T, U>").unwrap().unwrap(),
            IdlType::Defined("MyStruct".to_string())
        );
        assert_eq!(
            IdlType::from_str("MyStruct<T, U, const V: usize>")
                .unwrap()
                .unwrap(),
            IdlType::Defined("MyStruct".to_string())
        );
    }

    #[test]
    fn phantom_data() {
        assert_eq!(IdlType::from_str("PhantomData<T>").unwrap(), None);
    }

    #[test]
    fn array_phantom_data() {
        assert_eq!(IdlType::from_str("[PhantomData<T>; 16]").unwrap(), None);
    }

    #[test]
    fn option_phantom_data() {
        assert_eq!(IdlType::from_str("Option<PhantomData<T>>").unwrap(), None);
    }

    #[test]
    fn vector_phantom_data() {
        assert_eq!(IdlType::from_str("Vec<PhantomData<T>>").unwrap(), None);
    }

    #[test]
    fn phantom_pinned() {
        assert_eq!(IdlType::from_str("PhantomPinned").unwrap(), None);
    }

    #[test]
    fn array_phantom_pinned() {
        assert_eq!(IdlType::from_str("[PhantomPinned; 16]").unwrap(), None);
    }

    #[test]
    fn option_phantom_pinned() {
        assert_eq!(IdlType::from_str("Option<PhantomPinned>").unwrap(), None);
    }

    #[test]
    fn vector_phantom_pinned() {
        assert_eq!(IdlType::from_str("Vec<PhantomPinned>").unwrap(), None);
    }
}
