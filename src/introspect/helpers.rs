use crate::ir::ReferentialAction;

/// Map a SQL referential action string to the IR enum.
///
/// Returns [`ReferentialAction::NoAction`] when `s` is `None` (how
/// SQLite and some MySQL/MariaDB configurations report bare `REFERENCES`).
pub(crate) fn parse_referential_action(s: Option<&str>) -> ReferentialAction {
    match s {
        Some("CASCADE") => ReferentialAction::Cascade,
        Some("RESTRICT") => ReferentialAction::Restrict,
        Some("SET NULL") => ReferentialAction::SetNull,
        Some("SET DEFAULT") => ReferentialAction::SetDefault,
        Some("NO ACTION") => ReferentialAction::NoAction,
        _ => ReferentialAction::NoAction,
    }
}
