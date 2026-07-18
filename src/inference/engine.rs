use crate::inference::cardinality::{infer_cardinality, infer_inverse_cardinality};
use crate::inference::many_to_many::detect_many_to_many;
use crate::inference::naming::{relation_name_from_fk, singularize};
use crate::ir::{
    RelationCardinality, RelationInferenceStrategy, RelationOrigin, SchemaIR, SemanticRelationIR,
};

/// The main relation inference engine.
///
/// Consumes a `SchemaIR` and produces `SemanticRelationIR` entries
/// covering all relations in the schema — both explicit FK and inferred.
pub struct RelationInferenceEngine<'a> {
    schema: &'a SchemaIR,
}

impl<'a> RelationInferenceEngine<'a> {
    pub fn new(schema: &'a SchemaIR) -> Self {
        Self { schema }
    }

    /// Run the full inference pipeline and return all semantic relations.
    ///
    /// Order:
    /// 1. Detect many-to-many join tables
    /// 2. Infer cardinality for each FK relation
    /// 3. Generate relation names
    pub fn infer(&self) -> Vec<SemanticRelationIR> {
        let mut results = Vec::new();

        // Step 1: M2M detection
        let m2m = detect_many_to_many(&self.schema.tables, &self.schema.relations);
        results.extend(m2m);

        // Step 2: FK relations → SemanticRelationIR
        for relation in &self.schema.relations {
            let cardinality = infer_cardinality(relation, self.schema)
                .unwrap_or(RelationCardinality::ManyToOne);

            let inverse_cardinality = infer_inverse_cardinality(cardinality);

            // Build relation name
            let (relation_name, inverse_name) = match &relation.origin {
                RelationOrigin::ForeignKey => {
                    // Singularize the target table for the relation name
                    let name = singularize(&relation.to_table);
                    let inverse = Some(format!("{}s", singularize(&relation.from_table)));
                    (name, inverse)
                }
                RelationOrigin::Inferred { strategy } => match strategy {
                    RelationInferenceStrategy::Suffix => {
                        let name = relation_name_from_fk(
                            relation.from_columns.first().unwrap_or(&String::new()),
                            &relation.to_table,
                        );
                        let inverse = Some(format!("{}s", singularize(&relation.from_table)));
                        (name, inverse)
                    }
                    RelationInferenceStrategy::Prefix => {
                        let name = singularize(&relation.to_table);
                        let inverse = Some(format!("{}s", singularize(&relation.from_table)));
                        (name, inverse)
                    }
                    RelationInferenceStrategy::ExplicitMapping { .. } => {
                        let name = singularize(&relation.to_table);
                        let inverse = Some(format!("{}s", singularize(&relation.from_table)));
                        (name, inverse)
                    }
                },
            };

            results.push(SemanticRelationIR {
                relation: relation.clone(),
                cardinality,
                relation_name,
                inverse_name,
                inverse_cardinality,
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn make_schema(tables: Vec<TableIR>, relations: Vec<RelationIR>) -> SchemaIR {
        SchemaIR {
            ir_version: 1,
            metadata: Metadata::default(),
            tables,
            relations,
            enums: vec![],
        }
    }

    #[test]
    fn simple_fk_relation() {
        let posts = TableIR {
            name: "posts".into(),
            fields: vec![],
            constraints: vec![ConstraintIR {
                name: "pkey".into(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["id".into()],
                },
            }],
            comment: None,
            indexes: vec![],
        };

        let users = TableIR {
            name: "users".into(),
            fields: vec![],
            constraints: vec![ConstraintIR {
                name: "pkey".into(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["id".into()],
                },
            }],
            comment: None,
            indexes: vec![],
        };

        let relations = vec![RelationIR {
            from_table: "posts".into(),
            from_columns: vec!["user_id".into()],
            to_table: "users".into(),
            to_columns: vec!["id".into()],
            origin: RelationOrigin::ForeignKey,
        }];

        let schema = make_schema(vec![posts, users], relations);
        let engine = RelationInferenceEngine::new(&schema);
        let result = engine.infer();

        // Should have 1 FK relation (not M2M since there's no join table)
        assert_eq!(result.len(), 1);

        let rel = &result[0];
        assert_eq!(rel.relation.from_table, "posts");
        assert_eq!(rel.cardinality, RelationCardinality::ManyToOne);
        assert_eq!(rel.relation_name, "user");
        assert_eq!(rel.inverse_cardinality, Some(RelationCardinality::OneToMany));
    }

    #[test]
    fn one_to_one_via_unique_constraint() {
        let profiles = TableIR {
            name: "profiles".into(),
            fields: vec![],
            constraints: vec![
                ConstraintIR {
                    name: "pkey".into(),
                    kind: ConstraintKind::PrimaryKey {
                        columns: vec!["id".into()],
                    },
                },
                ConstraintIR {
                    name: "profiles_user_id_key".into(),
                    kind: ConstraintKind::Unique {
                        columns: vec!["user_id".into()],
                    },
                },
            ],
            comment: None,
            indexes: vec![],
        };

        let users = TableIR {
            name: "users".into(),
            fields: vec![],
            constraints: vec![ConstraintIR {
                name: "pkey".into(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["id".into()],
                },
            }],
            comment: None,
            indexes: vec![],
        };

        let relations = vec![RelationIR {
            from_table: "profiles".into(),
            from_columns: vec!["user_id".into()],
            to_table: "users".into(),
            to_columns: vec!["id".into()],
            origin: RelationOrigin::ForeignKey,
        }];

        let schema = make_schema(vec![profiles, users], relations);
        let engine = RelationInferenceEngine::new(&schema);
        let result = engine.infer();

        let rel = &result[0];
        assert_eq!(rel.cardinality, RelationCardinality::OneToOne);
        assert_eq!(rel.inverse_cardinality, Some(RelationCardinality::OneToOne));
        assert_eq!(rel.relation_name, "user");
    }

    #[test]
    fn many_to_many_via_join_table() {
        let join_table = TableIR {
            name: "user_roles".into(),
            fields: vec![],
            constraints: vec![
                ConstraintIR {
                    name: "pkey".into(),
                    kind: ConstraintKind::PrimaryKey {
                        columns: vec!["user_id".into(), "role_id".into()],
                    },
                },
                ConstraintIR {
                    name: "fk_user".into(),
                    kind: ConstraintKind::ForeignKey {
                        columns: vec!["user_id".into()],
                        referenced_table: "users".into(),
                        referenced_columns: vec!["id".into()],
                        on_delete: ReferentialAction::NoAction,
                        on_update: ReferentialAction::NoAction,
                        match_type: None,
                    },
                },
                ConstraintIR {
                    name: "fk_role".into(),
                    kind: ConstraintKind::ForeignKey {
                        columns: vec!["role_id".into()],
                        referenced_table: "roles".into(),
                        referenced_columns: vec!["id".into()],
                        on_delete: ReferentialAction::NoAction,
                        on_update: ReferentialAction::NoAction,
                        match_type: None,
                    },
                },
            ],
            comment: None,
            indexes: vec![],
        };

        let users = TableIR {
            name: "users".into(),
            fields: vec![],
            constraints: vec![ConstraintIR {
                name: "pkey".into(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["id".into()],
                },
            }],
            comment: None,
            indexes: vec![],
        };

        let roles = TableIR {
            name: "roles".into(),
            fields: vec![],
            constraints: vec![ConstraintIR {
                name: "pkey".into(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["id".into()],
                },
            }],
            comment: None,
            indexes: vec![],
        };

        let schema = make_schema(vec![users, roles, join_table], vec![]);
        let engine = RelationInferenceEngine::new(&schema);
        let result = engine.infer();

        // Should have 2 M2M relations (users↔roles in both directions)
        assert_eq!(result.len(), 2);

        let user_to_role = result.iter().find(|r| r.relation.from_table == "users").unwrap();
        assert_eq!(user_to_role.cardinality, RelationCardinality::ManyToMany);
        assert_eq!(user_to_role.relation_name, "role");

        let role_to_user = result.iter().find(|r| r.relation.from_table == "roles").unwrap();
        assert_eq!(role_to_user.cardinality, RelationCardinality::ManyToMany);
        assert_eq!(role_to_user.relation_name, "user");
    }
}
