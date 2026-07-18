/// Where a relation definition originated.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationOrigin {
    /// Derived from a real foreign key constraint in the database.
    ForeignKey,
    /// Inferred via a naming heuristic.
    Inferred {
        /// Which heuristic was used to discover this relation.
        strategy: RelationInferenceStrategy,
    },
}

/// Which heuristic produced an inferred relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationInferenceStrategy {
    /// Column ends with `_id` and a matching table was found (e.g. `user_id → users.id`).
    Suffix,
    /// Column starts with `id_` and a matching table was found (e.g. `id_users → users.id`).
    Prefix,
    /// Column matches a user-defined explicit mapping.
    ExplicitMapping,
}

/// Cardinality of a relation between two tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelationCardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Controls whether [`SchemaIR::from_tables`](crate::ir::SchemaIR::from_tables) attempts to infer foreign-key-like
/// relationships between tables.
///
/// Relation inference is always best-effort and does **not** query database
/// foreign key constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationStrategy {
    /// No relation inference — relations vec will be empty.
    Disabled,
    /// Infer relations using naming convention heuristic (column ends with `_id`).
    /// These are best-effort guesses, not verified foreign key constraints.
    NamingHeuristic,
}

/// A relationship between two tables.
///
/// Produced either from a real [`ForeignKey`](crate::ir::ConstraintKind::ForeignKey)
/// constraint or via the naming heuristic fallback.
///
/// One constraint = one [`RelationIR`]. Composite foreign keys appear as a
/// single entry with multiple columns in [`from_columns`](RelationIR::from_columns)
/// and [`to_columns`](RelationIR::to_columns).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationIR {
    /// Source table name.
    pub from_table: String,
    /// Source column(s) — multiple for composite foreign keys.
    pub from_columns: Vec<String>,
    /// Target table name.
    pub to_table: String,
    /// Target column(s) — multiple for composite foreign keys.
    pub to_columns: Vec<String>,
    /// How this relation was discovered.
    pub origin: RelationOrigin,
}

/// Application-facing semantic representation of a relation.
///
/// Wraps a database [`RelationIR`] with application-level semantics:
/// cardinality, relation names, and inverse information.
///
/// Generators consume `SemanticRelationIR` (not raw `RelationIR`) to
/// produce framework-specific code (Rust structs, GraphQL, TypeScript, etc.).
///
/// Traceability is preserved — the original database fact is always accessible:
///
/// ```rust,no_run
/// use neutrino_schema::*;
///
/// let semantic = SemanticRelationIR {
///     relation: RelationIR {
///         from_table: "posts".into(),
///         from_columns: vec!["user_id".into()],
///         to_table: "users".into(),
///         to_columns: vec!["id".into()],
///         origin: RelationOrigin::ForeignKey,
///     },
///     cardinality: RelationCardinality::ManyToOne,
///     relation_name: "user".into(),
///     inverse_name: Some("posts".into()),
///     inverse_cardinality: Some(RelationCardinality::OneToMany),
/// };
///
/// // Always traceable to the database fact:
/// assert_eq!(semantic.relation.origin, RelationOrigin::ForeignKey);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticRelationIR {
    /// The underlying database relation fact.
    pub relation: RelationIR,
    /// Cardinality from the source table's perspective.
    pub cardinality: RelationCardinality,
    /// Name of this relation (e.g. `"user"` for `Post.belongsTo User`).
    pub relation_name: String,
    /// Name of the inverse relation, if applicable (e.g. `"posts"` for `User.hasMany Posts`).
    pub inverse_name: Option<String>,
    /// Cardinality from the inverse table's perspective.
    pub inverse_cardinality: Option<RelationCardinality>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_origin_foreign_key_roundtrip() {
        let origin = RelationOrigin::ForeignKey;
        let json = serde_json::to_string(&origin).unwrap();
        assert_eq!(json, "\"foreignkey\"");
        let deser: RelationOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, origin);
    }

    #[test]
    fn relation_origin_inferred_suffix_roundtrip() {
        let origin = RelationOrigin::Inferred {
            strategy: RelationInferenceStrategy::Suffix,
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert_eq!(json, "{\"inferred\":{\"strategy\":\"suffix\"}}");
        let deser: RelationOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, origin);
    }

    #[test]
    fn relation_origin_inferred_prefix_roundtrip() {
        let origin = RelationOrigin::Inferred {
            strategy: RelationInferenceStrategy::Prefix,
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert_eq!(json, "{\"inferred\":{\"strategy\":\"prefix\"}}");
        let deser: RelationOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, origin);
    }

    #[test]
    fn relation_origin_inferred_explicit_mapping_roundtrip() {
        let origin = RelationOrigin::Inferred {
            strategy: RelationInferenceStrategy::ExplicitMapping,
        };
        let json = serde_json::to_string(&origin).unwrap();
        assert_eq!(json, "{\"inferred\":{\"strategy\":\"explicit_mapping\"}}");
        let deser: RelationOrigin = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, origin);
    }

    #[test]
    fn relation_cardinality_roundtrip() {
        let cases = vec![
            (RelationCardinality::OneToOne, "\"oneToOne\""),
            (RelationCardinality::OneToMany, "\"oneToMany\""),
            (RelationCardinality::ManyToOne, "\"manyToOne\""),
            (RelationCardinality::ManyToMany, "\"manyToMany\""),
        ];
        for (cardinality, expected_json) in cases {
            let json = serde_json::to_string(&cardinality).unwrap();
            assert_eq!(json, expected_json);
            let deser: RelationCardinality = serde_json::from_str(&json).unwrap();
            assert_eq!(deser, cardinality);
        }
    }

    #[test]
    fn relation_ir_roundtrip() {
        let relation = RelationIR {
            from_table: "posts".into(),
            from_columns: vec!["user_id".into()],
            to_table: "users".into(),
            to_columns: vec!["id".into()],
            origin: RelationOrigin::ForeignKey,
        };
        let json = serde_json::to_string_pretty(&relation).unwrap();
        let deser: RelationIR = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, relation);
    }

    #[test]
    fn semantic_relation_ir_roundtrip() {
        let semantic = SemanticRelationIR {
            relation: RelationIR {
                from_table: "posts".into(),
                from_columns: vec!["user_id".into()],
                to_table: "users".into(),
                to_columns: vec!["id".into()],
                origin: RelationOrigin::ForeignKey,
            },
            cardinality: RelationCardinality::ManyToOne,
            relation_name: "user".into(),
            inverse_name: Some("posts".into()),
            inverse_cardinality: Some(RelationCardinality::OneToMany),
        };
        let json = serde_json::to_string_pretty(&semantic).unwrap();
        let deser: SemanticRelationIR = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, semantic);
    }

    #[test]
    fn semantic_relation_ir_nullable_fields() {
        let semantic = SemanticRelationIR {
            relation: RelationIR {
                from_table: "orders".into(),
                from_columns: vec!["customer_id".into()],
                to_table: "customers".into(),
                to_columns: vec!["id".into()],
                origin: RelationOrigin::Inferred {
                    strategy: RelationInferenceStrategy::Suffix,
                },
            },
            cardinality: RelationCardinality::ManyToOne,
            relation_name: "customer".into(),
            inverse_name: None,
            inverse_cardinality: None,
        };
        let json = serde_json::to_string_pretty(&semantic).unwrap();
        let deser: SemanticRelationIR = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, semantic);
        assert!(deser.inverse_name.is_none());
        assert!(deser.inverse_cardinality.is_none());
    }

    #[test]
    fn composite_fk_relation_ir() {
        let relation = RelationIR {
            from_table: "order_items".into(),
            from_columns: vec!["order_id".into(), "product_id".into()],
            to_table: "orders".into(),
            to_columns: vec!["id".into(), "product_id".into()],
            origin: RelationOrigin::ForeignKey,
        };
        let json = serde_json::to_string_pretty(&relation).unwrap();
        let deser: RelationIR = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.from_columns.len(), 2);
        assert_eq!(deser.to_columns.len(), 2);
        assert_eq!(deser, relation);
    }
}
