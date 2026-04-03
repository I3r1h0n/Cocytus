use serde::{Serialize, Serializer};

fn is_false(v: &bool) -> bool {
    !v
}

fn serialize_hex_u32<S: Serializer>(value: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{:#X}", value))
}

fn serialize_hex_u64<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{:#X}", value))
}

fn serialize_hex_i64<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    if *value < 0 {
        serializer.serialize_str(&format!("-{:#X}", value.unsigned_abs()))
    } else {
        serializer.serialize_str(&format!("{:#X}", value))
    }
}

fn serialize_hex_option_u32<S: Serializer>(
    value: &Option<u32>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(v) => serializer.serialize_str(&format!("{:#X}", v)),
        None => serializer.serialize_none(),
    }
}

/// A single field within a struct/class/union
#[derive(Debug, Clone, Serialize)]
pub struct StructField {
    pub name: String,
    pub type_name: String,
    #[serde(serialize_with = "serialize_hex_u64")]
    pub offset: u64,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub access: String,
}

/// Resolved data from a FieldList chain
#[derive(Default)]
pub struct ResolvedFields {
    pub fields: Vec<StructField>,
    pub base_classes: Vec<BaseClassInfo>,
    pub methods: Vec<MethodInfo>,
    pub static_fields: Vec<StaticFieldInfo>,
    pub nested_types: Vec<NestedTypeInfo>,
}

/// A base class entry
#[derive(Debug, Clone, Serialize)]
pub struct BaseClassInfo {
    pub name: String,
    #[serde(serialize_with = "serialize_hex_u32")]
    pub offset: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_virtual: Option<bool>,
}

/// A method entry within a struct/class
#[derive(Debug, Clone, Serialize)]
pub struct MethodInfo {
    pub name: String,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub access: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub is_virtual: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub is_static: bool,
}

/// A static member field
#[derive(Debug, Clone, Serialize)]
pub struct StaticFieldInfo {
    pub name: String,
    pub type_name: String,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub access: String,
}

/// A nested type entry
#[derive(Debug, Clone, Serialize)]
pub struct NestedTypeInfo {
    pub name: String,
    pub type_name: String,
}

/// Information about a struct/class/union parsed from a PDB
#[derive(Debug, Clone, Serialize)]
pub struct StructInfo {
    pub name: String,
    pub kind: String,
    #[serde(serialize_with = "serialize_hex_u64")]
    pub size: u64,
    pub fields: Vec<StructField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub base_classes: Vec<BaseClassInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<MethodInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub static_fields: Vec<StaticFieldInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nested_types: Vec<NestedTypeInfo>,
}

/// A single enum variant value
#[derive(Debug, Clone, Serialize)]
pub struct EnumValue {
    pub name: String,
    #[serde(serialize_with = "serialize_hex_i64")]
    pub value: i64,
}

/// Information about an enumeration parsed from a PDB
#[derive(Debug, Clone, Serialize)]
pub struct EnumInfo {
    pub name: String,
    pub underlying_type: String,
    pub values: Vec<EnumValue>,
}

/// Pre-computed function signature resolved during type iteration
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub return_type: Option<String>,
    pub parameters: Vec<String>,
    pub calling_convention: String,
}

/// Information about a function symbol parsed from a PDB
#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    /// Relative virtual address
    #[serde(serialize_with = "serialize_hex_u32")]
    pub rva: u32,
    #[serde(serialize_with = "serialize_hex_option_u32", skip_serializing_if = "Option::is_none")]
    pub len: Option<u32>,
    pub return_type: Option<String>,
    pub parameters: Vec<String>,
    pub calling_convention: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub is_global: bool,
}
