use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

pub mod file;
pub mod pda;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Idl {
    pub version: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub docs: Option<Vec<String>>,
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

impl Idl {
    pub fn deserialize_account_to_string(
        &self,
        account_name: &str,
        data: &mut &[u8],
    ) -> Result<String, anyhow::Error> {
        let mut deserialized_fields: HashMap<String, String> = HashMap::new();

        let account_type = &self
            .accounts
            .iter()
            .chain(self.types.iter())
            .find(|account_type| account_type.name == account_name)
            .ok_or(anyhow::anyhow!(
                "Struct/Enum named {} not found in IDL.",
                account_name.clone()
            ))?
            .ty;

        match account_type {
            IdlTypeDefinitionTy::Struct { fields } => {
                for field in fields {
                    deserialized_fields.insert(
                        field.name.clone(),
                        field.ty.deserialize_to_string(data, &self)?,
                    );
                }
            }
            IdlTypeDefinitionTy::Enum { variants } => {
                todo!("{:?}", variants);
            }
        }

        Ok(format!("{} {:#?}", account_name, deserialized_fields))
    }
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
    pub docs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pda: Option<IdlPda>,
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
    Bytes,
    String,
    PublicKey,
    Defined(String),
    Option(Box<IdlType>),
    Vec(Box<IdlType>),
    Array(Box<IdlType>, usize),
}

impl std::str::FromStr for IdlType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.to_string();
        fn array_from_str(inner: &str) -> IdlType {
            match inner.strip_suffix(']') {
                None => {
                    let (raw_type, raw_length) = inner.rsplit_once(';').unwrap();
                    let ty = IdlType::from_str(raw_type).unwrap();
                    let len = raw_length.replace('_', "").parse::<usize>().unwrap();
                    IdlType::Array(Box::new(ty), len)
                }
                Some(nested_inner) => array_from_str(&nested_inner[1..]),
            }
        }
        s.retain(|c| !c.is_whitespace());

        let r = match s.as_str() {
            "bool" => IdlType::Bool,
            "u8" => IdlType::U8,
            "i8" => IdlType::I8,
            "u16" => IdlType::U16,
            "i16" => IdlType::I16,
            "u32" => IdlType::U32,
            "i32" => IdlType::I32,
            "f32" => IdlType::F32,
            "u64" => IdlType::U64,
            "i64" => IdlType::I64,
            "f64" => IdlType::F64,
            "u128" => IdlType::U128,
            "i128" => IdlType::I128,
            "Vec<u8>" => IdlType::Bytes,
            "String" | "&str" => IdlType::String,
            "Pubkey" => IdlType::PublicKey,
            _ => match s.to_string().strip_prefix("Option<") {
                None => match s.to_string().strip_prefix("Vec<") {
                    None => {
                        if s.to_string().starts_with('[') {
                            array_from_str(&s)
                        } else {
                            IdlType::Defined(s.to_string())
                        }
                    }
                    Some(inner) => {
                        let inner_ty = Self::from_str(
                            inner
                                .strip_suffix('>')
                                .ok_or_else(|| anyhow::anyhow!("Invalid option"))?,
                        )?;
                        IdlType::Vec(Box::new(inner_ty))
                    }
                },
                Some(inner) => {
                    let inner_ty = Self::from_str(
                        inner
                            .strip_suffix('>')
                            .ok_or_else(|| anyhow::anyhow!("Invalid option"))?,
                    )?;
                    IdlType::Option(Box::new(inner_ty))
                }
            },
        };
        Ok(r)
    }
}

impl IdlType {
    pub fn deserialize_to_string(
        &self,
        data: &mut &[u8],
        parent_idl: &Idl,
    ) -> Result<String, anyhow::Error> {
        if data.len() == 0 {
            return Err(anyhow::anyhow!("Unable to parse from empty bytes"));
        }

        Ok(match self {
            IdlType::Bool => <bool as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::U8 => <u8 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::I8 => <i8 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::U16 => <u16 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::I16 => <i16 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::U32 => <u32 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::I32 => <i32 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::F32 => <f32 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::U64 => <u64 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::I64 => <i64 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::F64 => <f64 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::U128 => <u128 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::I128 => <i128 as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::Bytes => format!("{:?}", <Vec<u8> as BorshDeserialize>::deserialize(data)?),
            IdlType::String => <String as BorshDeserialize>::deserialize(data)?,
            IdlType::PublicKey => <Pubkey as BorshDeserialize>::deserialize(data)?.to_string(),
            IdlType::Defined(type_name) => {
                parent_idl.deserialize_account_to_string(&type_name, data)?
            }
            IdlType::Option(ty) => {
                let is_present = <u8 as BorshDeserialize>::deserialize(data)?;

                if is_present == 0 {
                    "None".to_string()
                } else {
                    ty.deserialize_to_string(data, parent_idl)?
                }
            }
            IdlType::Vec(ty) => {
                let size: usize = <u32 as BorshDeserialize>::deserialize(data)?
                    .try_into()
                    .unwrap();

                let mut vec_data: Vec<String> = Vec::with_capacity(size);

                for _ in 0..size {
                    vec_data.push(ty.deserialize_to_string(data, parent_idl)?.to_string());
                }

                format!("{:?}", vec_data)
            }
            IdlType::Array(ty, size) => {
                let mut array_data: Vec<String> = Vec::with_capacity(*size);

                for _ in 0..*size {
                    array_data.push(ty.deserialize_to_string(data, parent_idl)?.to_string());
                }

                format!("{:?}", array_data)
            }
        })
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
}
