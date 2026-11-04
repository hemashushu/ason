use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Char(char),
    String(String),
    DateTime(DateTime<FixedOffset>),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Tuple(Vec<Value>),
    Object(Vec<(String, Value)>),
    Enumeration {
        name: String,
        variant: String,
        payload: VariantValue,
    },
    Option(Option<Box<Value>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariantValue {
    Unit,
    Newtype(Box<Value>),
    Tuple(Vec<Value>),
    Struct(Vec<(String, Value)>),
}
