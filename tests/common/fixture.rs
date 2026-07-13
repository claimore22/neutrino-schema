#![allow(dead_code)]
/// Load a fixture SQL file from `tests/fixtures/{name}/schema.sql`.
pub fn load_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{}/schema.sql",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to load fixture '{name}' from {path}: {e}"))
}
