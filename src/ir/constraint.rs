use crate::ir::RelationSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintIR {
    pub name: String,
    pub kind: ConstraintKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintKind {
    PrimaryKey { columns: Vec<String> },
    ForeignKey {
        columns: Vec<String>,
        referenced_table: String,
        referenced_columns: Vec<String>,
        on_delete: ReferentialAction,
        on_update: ReferentialAction,
        match_type: Option<MatchType>,
    },
    Unique { columns: Vec<String> },
    Check { expression: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferentialAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Full,
    Partial,
    Simple,
}

impl ConstraintIR {
    pub fn fk_relations(&self, from_table: &str) -> Vec<super::RelationIR> {
        match &self.kind {
            ConstraintKind::ForeignKey {
                columns,
                referenced_table,
                referenced_columns,
                ..
            } => columns
                .iter()
                .zip(referenced_columns.iter())
                .map(|(from_col, to_col)| super::RelationIR {
                    from_table: from_table.to_string(),
                    from_field: from_col.clone(),
                    to_table: referenced_table.clone(),
                    to_field: to_col.clone(),
                    source: RelationSource::ForeignKey(self.name.clone()),
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl std::fmt::Display for ReferentialAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferentialAction::NoAction => write!(f, "NO ACTION"),
            ReferentialAction::Restrict => write!(f, "RESTRICT"),
            ReferentialAction::Cascade => write!(f, "CASCADE"),
            ReferentialAction::SetNull => write!(f, "SET NULL"),
            ReferentialAction::SetDefault => write!(f, "SET DEFAULT"),
        }
    }
}
