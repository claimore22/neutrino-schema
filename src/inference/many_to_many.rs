use std::collections::HashMap;

use crate::ir::{ConstraintKind, RelationCardinality, RelationIR, SemanticRelationIR, TableIR};
use crate::inference::naming::singularize;

/// Detect many-to-many join tables and produce `SemanticRelationIR` entries.
///
/// A join table is detected when:
/// - Its PK consists of exactly 2 columns
/// - Both PK columns are FKs to different tables
///
/// For each detected join table, two `SemanticRelationIR` entries are produced
/// (one for each direction).
pub fn detect_many_to_many(
    tables: &[TableIR],
    existing_relations: &[RelationIR],
) -> Vec<SemanticRelationIR> {
    let mut results = Vec::new();

    // Build a map: table_name → Vec<ForeignKey { columns, referenced_table, referenced_columns }>
    let fk_map = build_fk_map(tables);

    for table in tables {
        // Check if this table is a join table
        let Some(join_columns) = get_join_table_columns(table) else {
            continue;
        };

        // Each PK column must be a FK to a different table
        let mut fk_refs: Vec<(Vec<String>, String, Vec<String>)> = Vec::new();
        let mut seen_targets = std::collections::HashSet::new();

        for pk_col in &join_columns {
            if let Some(fk_list) = fk_map.get(&(&table.name, pk_col.as_str())) {
                for fk in fk_list {
                    if seen_targets.insert(fk.1.clone()) {
                        fk_refs.push(fk.clone());
                    }
                }
            }
        }

        // Need exactly 2 FK references to different tables
        if fk_refs.len() != 2 {
            continue;
        }

        let (left_cols, left_table, left_target_cols) = &fk_refs[0];
        let (right_cols, right_table, right_target_cols) = &fk_refs[1];

        // Skip if this join table is already covered by a relation in existing_relations
        let already_covered = existing_relations.iter().any(|r| {
            r.from_table == table.name || r.to_table == table.name
        });
        if already_covered {
            continue;
        }

        // Left side: left_table <-> join_table
        let left_name = singularize(left_table);
        let right_name = singularize(right_table);
        let left_plural = format!("{}s", singularize(left_table));
        let right_plural = format!("{}s", singularize(right_table));

        // Left relation
        results.push(SemanticRelationIR {
            relation: RelationIR {
                from_table: left_table.clone(),
                from_columns: left_cols.clone(),
                to_table: right_table.clone(),
                to_columns: right_target_cols.clone(),
                origin: crate::ir::RelationOrigin::ForeignKey,
            },
            cardinality: RelationCardinality::ManyToMany,
            relation_name: right_name.clone(),
            inverse_name: Some(left_plural),
            inverse_cardinality: Some(RelationCardinality::ManyToMany),
        });

        // Right relation
        results.push(SemanticRelationIR {
            relation: RelationIR {
                from_table: right_table.clone(),
                from_columns: right_cols.clone(),
                to_table: left_table.clone(),
                to_columns: left_target_cols.clone(),
                origin: crate::ir::RelationOrigin::ForeignKey,
            },
            cardinality: RelationCardinality::ManyToMany,
            relation_name: left_name,
            inverse_name: Some(right_plural),
            inverse_cardinality: Some(RelationCardinality::ManyToMany),
        });
    }

    results
}

/// Get the PK columns of a table, if the PK is composite (exactly 2 columns).
fn get_join_table_columns(table: &TableIR) -> Option<Vec<String>> {
    let pk = table.primary_key()?;
    if let ConstraintKind::PrimaryKey { columns } = &pk.kind {
        if columns.len() == 2 {
            return Some(columns.clone());
        }
    }
    None
}

/// Build a map of (table_name, column_name) → Vec<(from_columns, referenced_table, referenced_columns)>.
fn build_fk_map(tables: &[TableIR]) -> HashMap<(&str, &str), Vec<(Vec<String>, String, Vec<String>)>> {
    let mut map: HashMap<(&str, &str), Vec<(Vec<String>, String, Vec<String>)>> = HashMap::new();

    for table in tables {
        for constraint in &table.constraints {
            if let ConstraintKind::ForeignKey {
                columns,
                referenced_table,
                referenced_columns,
                ..
            } = &constraint.kind
            {
                for col in columns {
                    map.entry((table.name.as_str(), col.as_str()))
                        .or_default()
                        .push((
                            columns.clone(),
                            referenced_table.clone(),
                            referenced_columns.clone(),
                        ));
                }
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn make_fk_constraint(columns: Vec<&str>, ref_table: &str, ref_columns: Vec<&str>) -> ConstraintIR {
        ConstraintIR {
            name: format!("fk_{}", columns[0]),
            kind: ConstraintKind::ForeignKey {
                columns: columns.into_iter().map(String::from).collect(),
                referenced_table: ref_table.into(),
                referenced_columns: ref_columns.into_iter().map(String::from).collect(),
                on_delete: ReferentialAction::NoAction,
                on_update: ReferentialAction::NoAction,
                match_type: None,
            },
        }
    }

    fn make_pk_constraint(columns: Vec<&str>) -> ConstraintIR {
        ConstraintIR {
            name: "pkey".into(),
            kind: ConstraintKind::PrimaryKey {
                columns: columns.into_iter().map(String::from).collect(),
            },
        }
    }

    #[test]
    fn detects_join_table() {
        let join_table = TableIR {
            name: "user_roles".into(),
            fields: vec![],
            constraints: vec![
                make_pk_constraint(vec!["user_id", "role_id"]),
                make_fk_constraint(vec!["user_id"], "users", vec!["id"]),
                make_fk_constraint(vec!["role_id"], "roles", vec!["id"]),
            ],
            comment: None,
            indexes: vec![],
        };

        let users = TableIR {
            name: "users".into(),
            fields: vec![],
            constraints: vec![make_pk_constraint(vec!["id"])],
            comment: None,
            indexes: vec![],
        };

        let roles = TableIR {
            name: "roles".into(),
            fields: vec![],
            constraints: vec![make_pk_constraint(vec!["id"])],
            comment: None,
            indexes: vec![],
        };

        let result = detect_many_to_many(&[users, roles, join_table], &[]);
        assert_eq!(result.len(), 2);

        // Check one direction
        let user_to_role = result.iter().find(|r| r.relation.from_table == "users").unwrap();
        assert_eq!(user_to_role.cardinality, RelationCardinality::ManyToMany);
        assert_eq!(user_to_role.relation_name, "role");
        assert_eq!(user_to_role.inverse_name, Some("users".into()));
        assert_eq!(user_to_role.inverse_cardinality, Some(RelationCardinality::ManyToMany));

        // Check other direction
        let role_to_user = result.iter().find(|r| r.relation.from_table == "roles").unwrap();
        assert_eq!(role_to_user.cardinality, RelationCardinality::ManyToMany);
        assert_eq!(role_to_user.relation_name, "user");
    }

    #[test]
    fn non_join_table_ignored() {
        let table = TableIR {
            name: "posts".into(),
            fields: vec![],
            constraints: vec![
                make_pk_constraint(vec!["id"]),
                make_fk_constraint(vec!["user_id"], "users", vec!["id"]),
            ],
            comment: None,
            indexes: vec![],
        };

        let result = detect_many_to_many(&[table], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn join_table_with_three_pks_ignored() {
        let table = TableIR {
            name: "complex_join".into(),
            fields: vec![],
            constraints: vec![
                make_pk_constraint(vec!["a_id", "b_id", "c_id"]),
                make_fk_constraint(vec!["a_id"], "a", vec!["id"]),
                make_fk_constraint(vec!["b_id"], "b", vec!["id"]),
                make_fk_constraint(vec!["c_id"], "c", vec!["id"]),
            ],
            comment: None,
            indexes: vec![],
        };

        let result = detect_many_to_many(&[table], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn already_covered_by_existing_relation_skipped() {
        let join_table = TableIR {
            name: "user_roles".into(),
            fields: vec![],
            constraints: vec![
                make_pk_constraint(vec!["user_id", "role_id"]),
                make_fk_constraint(vec!["user_id"], "users", vec!["id"]),
                make_fk_constraint(vec!["role_id"], "roles", vec!["id"]),
            ],
            comment: None,
            indexes: vec![],
        };

        let existing = vec![RelationIR {
            from_table: "users".into(),
            from_columns: vec!["id".into()],
            to_table: "user_roles".into(),
            to_columns: vec!["user_id".into()],
            origin: RelationOrigin::ForeignKey,
        }];

        let result = detect_many_to_many(&[join_table], &existing);
        assert!(result.is_empty());
    }
}
