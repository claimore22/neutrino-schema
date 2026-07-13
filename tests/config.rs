#![cfg(feature = "cli")]

use std::collections::HashMap;
use std::path::PathBuf;

use neutrino_schema::config::{DatabaseConfig, DatabaseProvider, ProjectConfig};

#[test]
fn default_config_version_is_1() {
    let config = ProjectConfig::default();
    assert_eq!(config.version, 1);
}

#[test]
fn default_config_empty_databases() {
    let config = ProjectConfig::default();
    assert!(config.databases.is_empty());
}

#[test]
fn default_config_has_default_generator() {
    let config = ProjectConfig::default();
    assert_eq!(config.generator.output_dir, PathBuf::from("./src/entities"));
    assert_eq!(config.generator.module_name, "types");
}

#[test]
fn default_database_config() {
    let db = DatabaseConfig::default();
    assert!(db.url.is_none());
    assert!(db.provider.is_none());
    assert!(db.output.is_none());
}

#[test]
fn round_trip() {
    let mut config = ProjectConfig::default();
    config.databases.insert(
        "default".into(),
        DatabaseConfig {
            url: Some("postgres://localhost/test".into()),
            provider: Some(DatabaseProvider::Postgres),
            output: Some(PathBuf::from("custom_output")),
        },
    );
    config.generator.output_dir = PathBuf::from("./out");

    let toml_str = toml::to_string_pretty(&config).expect("serialize");
    let parsed: ProjectConfig = toml::from_str(&toml_str).expect("deserialize");

    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.generator.output_dir, PathBuf::from("./out"));

    let db = parsed.databases.get("default").expect("default db");
    assert_eq!(db.url.as_deref(), Some("postgres://localhost/test"));
    assert_eq!(db.provider, Some(DatabaseProvider::Postgres));
    assert_eq!(
        db.output.as_deref(),
        Some(PathBuf::from("custom_output").as_path())
    );
}

#[test]
fn save_updates_existing_url() {
    // Simulate the save operation: load existing config, modify URL, write back.
    let original = r#"version = 1

[databases.default]
url = "postgres://localhost/original"

[generator]
output = "src/types"
"#;

    let mut config: ProjectConfig = toml::from_str(original).expect("parse original");
    let db = config.databases.get_mut("default").expect("default db");
    db.url = Some("postgres://localhost/updated".into());

    let updated = toml::to_string_pretty(&config).expect("serialize");
    let reparsed: ProjectConfig = toml::from_str(&updated).expect("reparse");

    assert_eq!(
        reparsed.databases["default"].url.as_deref(),
        Some("postgres://localhost/updated")
    );
    assert_eq!(reparsed.generator.output_dir, PathBuf::from("src/types"));
}

#[test]
fn detect_provider_from_url() {
    use neutrino_schema::config::detect_provider;

    assert_eq!(
        detect_provider("postgres://localhost/db"),
        Some(DatabaseProvider::Postgres)
    );
    assert_eq!(
        detect_provider("postgresql://localhost/db"),
        Some(DatabaseProvider::Postgres)
    );
    assert_eq!(
        detect_provider("mysql://localhost/db"),
        Some(DatabaseProvider::MySql)
    );
    assert_eq!(
        detect_provider("mariadb://localhost/db"),
        Some(DatabaseProvider::MySql)
    );
    assert_eq!(
        detect_provider("sqlite:./db.sqlite"),
        Some(DatabaseProvider::Sqlite)
    );
    assert_eq!(
        detect_provider("sqlite://localhost/db"),
        Some(DatabaseProvider::Sqlite)
    );
    assert_eq!(detect_provider("oracle://localhost/db"), None);
    assert_eq!(detect_provider(""), None);
}

#[test]
fn database_provider_display_name() {
    assert_eq!(DatabaseProvider::Postgres.display_name(), "PostgreSQL");
    assert_eq!(DatabaseProvider::MySql.display_name(), "MySQL");
    assert_eq!(DatabaseProvider::Sqlite.display_name(), "SQLite");
}

#[test]
fn generator_output_serde_rename() {
    // Verify the TOML field is "output", not "output_dir"
    let toml_str = r#"
        [generator]
        output = "custom_path"
        "#;

    let config: ProjectConfig = toml::from_str(toml_str).expect("parse");
    assert_eq!(config.generator.output_dir, PathBuf::from("custom_path"));
}

#[test]
fn partial_generator_defaults() {
    let toml_str = r#"
        [generator]
        output = "custom_path"
        "#;

    let config: ProjectConfig = toml::from_str(toml_str).expect("parse");
    assert_eq!(config.generator.module_name, "types");
}

#[test]
fn empty_config_is_valid() {
    let config = ProjectConfig::default();
    let toml_str = toml::to_string_pretty(&config).expect("serialize");
    let reparsed: ProjectConfig = toml::from_str(&toml_str).expect("deserialize");
    assert_eq!(reparsed.version, 1);
    assert!(reparsed.databases.is_empty());
}

#[test]
fn multiple_databases() {
    let mut config = ProjectConfig::default();
    config.databases = HashMap::from([
        (
            "default".into(),
            DatabaseConfig {
                url: Some("postgres://localhost/app".into()),
                provider: None,
                output: None,
            },
        ),
        (
            "analytics".into(),
            DatabaseConfig {
                url: Some("mysql://localhost/analytics".into()),
                provider: Some(DatabaseProvider::MySql),
                output: Some(PathBuf::from("src/types/analytics")),
            },
        ),
    ]);

    let toml_str = toml::to_string_pretty(&config).expect("serialize");
    let parsed: ProjectConfig = toml::from_str(&toml_str).expect("deserialize");

    assert_eq!(parsed.databases.len(), 2);

    let default = &parsed.databases["default"];
    assert_eq!(default.url.as_deref(), Some("postgres://localhost/app"));

    let analytics = &parsed.databases["analytics"];
    assert_eq!(
        analytics.url.as_deref(),
        Some("mysql://localhost/analytics")
    );
    assert_eq!(analytics.provider, Some(DatabaseProvider::MySql));
}
