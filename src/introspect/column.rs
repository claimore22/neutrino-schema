use crate::types::PgType;

#[derive(Debug, Clone)]
pub struct Column {
    pub table_name: String,
    pub column_name: String,
    pub data_type: PgType,
    pub nullable: bool,
}
