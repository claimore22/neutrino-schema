use crate::types::DbType;

#[derive(Debug)]
pub struct FieldIR {
    pub name: String,
    pub ty: DbType,
    pub nullable: bool,
    /// Raw SQL type name, only used for debug/CLI display. Never consulted by type pipeline.
    pub raw_type: String,
}
