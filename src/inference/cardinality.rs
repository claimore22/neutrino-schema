use crate::ir::{
    ConstraintKind, IndexEntryIR, RelationCardinality, RelationIR, SchemaIR, TableIR,
};

/// Infer cardinality for a single relation using schema metadata.
///
/// Checks (in order):
/// 1. Is this a join table (composite PK of 2 FKs)? → skip (handled by M2M detector)
/// 2. Is the FK column unique (constraint or unique index)? → OneToOne
/// 3. Default → ManyToOne
pub fn infer_cardinality(
    relation: &RelationIR,
    schema: &SchemaIR,
) -> Option<RelationCardinality> {
    let from_table = schema.table(&relation.from_table)?;

    // Skip composite FKs — they're typically join tables (handled by M2M)
    if relation.from_columns.len() > 1 {
        return Some(RelationCardinality::ManyToOne);
    }

    let fk_column = relation.from_columns.first()?;

    // Check if FK column is unique → OneToOne
    if is_column_unique(from_table, fk_column) {
        return Some(RelationCardinality::OneToOne);
    }

    // Default → ManyToOne
    Some(RelationCardinality::ManyToOne)
}

/// Infer the inverse cardinality for the other side of a relation.
pub fn infer_inverse_cardinality(cardinality: RelationCardinality) -> Option<RelationCardinality> {
    match cardinality {
        RelationCardinality::ManyToOne => Some(RelationCardinality::OneToMany),
        RelationCardinality::OneToMany => Some(RelationCardinality::ManyToOne),
        RelationCardinality::OneToOne => Some(RelationCardinality::OneToOne),
        // M2M inverse is also M2M — but caller should handle this separately
        RelationCardinality::ManyToMany => Some(RelationCardinality::ManyToMany),
    }
}

/// Check if a column is unique in a table (via UNIQUE constraint or unique index).
fn is_column_unique(table: &TableIR, column: &str) -> bool {
    // Check UNIQUE constraints
    for constraint in &table.constraints {
        if let ConstraintKind::Unique { columns } = &constraint.kind {
            if columns.len() == 1 && columns[0] == column {
                return true;
            }
        }
    }

    // Check unique indexes
    for index in &table.indexes {
        if !index.unique {
            continue;
        }
        // Only check single-column unique indexes
        if index.entries.len() != 1 {
            continue;
        }
        match &index.entries[0] {
            IndexEntryIR::Column { name, .. } => {
                if name == column {
                    return true;
                }
            }
            IndexEntryIR::Expression { .. } => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn make_table(name: &str, constraints: Vec<ConstraintIR>, indexes: Vec<IndexIR>) -> TableIR {
        TableIR {
            name: name.into(),
            fields: vec![],
            constraints,
            comment: None,
            indexes,
        }
    }

    fn make_fk_relation(from_table: &str, from_col: &str, to_table: &str) -> RelationIR {
        RelationIR {
            from_table: from_table.into(),
            from_columns: vec![from_col.into()],
            to_table: to_table.into(),
            to_columns: vec!["id".into()],
            origin: RelationOrigin::ForeignKey,
        }
    }

    fn make_schema(tables: Vec<TableIR>) -> SchemaIR {
        SchemaIR {
            ir_version: 1,
            metadata: Metadata::default(),
            tables,
            relations: vec![],
            enums: vec![],
        }
    }

    #[test]
    fn fk_not_unique_is_many_to_one() {
        let table = make_table("posts", vec![], vec![]);
        let schema = make_schema(vec![table]);
        let relation = make_fk_relation("posts", "user_id", "users");
        assert_eq!(
            infer_cardinality(&relation, &schema),
            Some(RelationCardinality::ManyToOne)
        );
    }

    #[test]
    fn fk_with_unique_constraint_is_one_to_one() {
        let table = make_table(
            "profiles",
            vec![ConstraintIR {
                name: "profiles_user_id_key".into(),
                kind: ConstraintKind::Unique {
                    columns: vec!["user_id".into()],
                },
            }],
            vec![],
        );
        let schema = make_schema(vec![table]);
        let relation = make_fk_relation("profiles", "user_id", "users");
        assert_eq!(
            infer_cardinality(&relation, &schema),
            Some(RelationCardinality::OneToOne)
        );
    }

    #[test]
    fn fk_with_unique_index_is_one_to_one() {
        let table = make_table(
            "profiles",
            vec![],
            vec![IndexIR {
                name: "profiles_user_id_key".into(),
                table_name: "profiles".into(),
                entries: vec![IndexEntryIR::Column {
                    name: "user_id".into(),
                    descending: false,
                }],
                unique: true,
                kind: IndexKind::BTree,
                predicate: None,
            }],
        );
        let schema = make_schema(vec![table]);
        let relation = make_fk_relation("profiles", "user_id", "users");
        assert_eq!(
            infer_cardinality(&relation, &schema),
            Some(RelationCardinality::OneToOne)
        );
    }

    #[test]
    fn composite_fk_is_many_to_one() {
        let relation = RelationIR {
            from_table: "order_items".into(),
            from_columns: vec!["order_id".into(), "product_id".into()],
            to_table: "orders".into(),
            to_columns: vec!["id".into(), "product_id".into()],
            origin: RelationOrigin::ForeignKey,
        };
        let table = make_table("order_items", vec![], vec![]);
        let schema = make_schema(vec![table]);
        assert_eq!(
            infer_cardinality(&relation, &schema),
            Some(RelationCardinality::ManyToOne)
        );
    }

    #[test]
    fn inverse_cardinality_mapping() {
        assert_eq!(
            infer_inverse_cardinality(RelationCardinality::ManyToOne),
            Some(RelationCardinality::OneToMany)
        );
        assert_eq!(
            infer_inverse_cardinality(RelationCardinality::OneToMany),
            Some(RelationCardinality::ManyToOne)
        );
        assert_eq!(
            infer_inverse_cardinality(RelationCardinality::OneToOne),
            Some(RelationCardinality::OneToOne)
        );
        assert_eq!(
            infer_inverse_cardinality(RelationCardinality::ManyToMany),
            Some(RelationCardinality::ManyToMany)
        );
    }

    #[test]
    fn non_unique_multi_column_constraint_does_not_affect() {
        let table = make_table(
            "posts",
            vec![ConstraintIR {
                name: "posts_composite_uniq".into(),
                kind: ConstraintKind::Unique {
                    columns: vec!["user_id".into(), "category_id".into()],
                },
            }],
            vec![],
        );
        let schema = make_schema(vec![table]);
        let relation = make_fk_relation("posts", "user_id", "users");
        // Multi-column unique doesn't make a single column unique
        assert_eq!(
            infer_cardinality(&relation, &schema),
            Some(RelationCardinality::ManyToOne)
        );
    }
}
