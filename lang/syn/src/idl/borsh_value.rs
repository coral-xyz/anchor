pub enum IdlValue {
    Bool,
    U8,
    Defined(Box<IdlTypeDefinitionValue>),
}

pub enum IdlTypeDefinitionValue {
    Struct { fields: Vec<IdlValue> },
}
