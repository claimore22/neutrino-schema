use crate::ir::FieldIR;

#[derive(Debug)]
pub struct TableIR {
    pub name: String,
    pub fields: Vec<FieldIR>,
}
