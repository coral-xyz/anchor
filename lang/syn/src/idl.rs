use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Idl {
    pub version: String,
    pub name: String,
    pub instructions: Vec<IdlIx>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub state: Option<IdlState>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub accounts: Vec<IdlTypeDef>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub types: Vec<IdlTypeDef>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub errors: Option<Vec<IdlErrorCode>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlState {
    #[serde(rename = "struct")]
    pub strct: IdlTypeDef,
    pub methods: Vec<IdlStateMethod>,
}

pub type IdlStateMethod = IdlIx;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlIx {
    pub name: String,
    pub accounts: Vec<IdlAccountItem>,
    pub args: Vec<IdlField>,
}

// A single struct deriving `Accounts`.
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

// A single field in the accounts struct.
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
pub struct IdlTypeDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: IdlTypeDefTy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum IdlTypeDefTy {
    Struct { fields: Vec<IdlField> },
    Enum { variants: Vec<EnumVariant> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnumVariant {
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

/// Structure to serialize the map variant as a struct for easier use in TypeScript.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapStruct {
    pub key: IdlType,
    pub value: IdlType,
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
    Map(Box<MapStruct>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlTypePublicKey;

impl std::str::FromStr for IdlType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Eliminate whitespace.
        let mut s = s.to_string();
        s.retain(|c| !c.is_whitespace());

        let r = match s.as_str() {
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
            _ => {
                if let Some(inner) = s.to_string().strip_prefix("Option<") {
                    let inner_ty = Self::from_str(
                        inner
                            .strip_suffix(">")
                            .ok_or_else(|| anyhow::anyhow!("Invalid option"))?,
                    )?;
                    IdlType::Option(Box::new(inner_ty))
                } else if let Some(inner) = s.to_string().strip_prefix("Vec<") {
                    let inner_ty = Self::from_str(
                        inner
                            .strip_suffix(">")
                            .ok_or_else(|| anyhow::anyhow!("Invalid vector"))?,
                    )?;
                    IdlType::Vec(Box::new(inner_ty))
                } else if let Some(inner) = s.to_string().strip_prefix("HashMap<") {
                    let inner = inner
                        .strip_suffix(">")
                        .ok_or_else(|| anyhow::anyhow!("Invalid HashMap"))?;

                    let mut types = inner.split(",");
                    let key = types
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Invalid HashMap key"))?
                        .trim();
                    let value = types
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Invalid HashMap value"))?
                        .trim();

                    if types.next().is_some() {
                        return Err(anyhow::anyhow!("Invalid HashMap: must be two types"));
                    }

                    IdlType::Map(Box::new(MapStruct {
                        key: Self::from_str(key)?,
                        value: Self::from_str(value)?,
                    }))
                } else {
                    IdlType::Defined(s.to_string())
                }
            }
        };
        Ok(r)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdlErrorCode {
    pub code: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub msg: Option<String>,
}
