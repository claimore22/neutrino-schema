/// A reference to a named enum defined in [`SchemaIR::enums`](crate::SchemaIR).
///
/// By the time this reaches [`DbType::Enum`](crate::DbType::Enum), the compiler has already resolved
/// the database enum to a stable Rust name.  The codegen uses `rust_name`
/// directly to produce `super::enums::Status`-style references.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct EnumRef {
    /// The PascalCase Rust identifier for this enum type.
    pub rust_name: String,
}
