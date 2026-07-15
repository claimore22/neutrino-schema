use std::time::SystemTime;

use crate::config::DatabaseProvider;

/// Metadata about a SchemaIR document — provenance, generation time, and
/// database provider.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The database provider that produced this IR, if any.
    pub provider: Option<DatabaseProvider>,
    /// ISO 8601 / RFC 3339 timestamp of when this IR was generated.
    pub generated_at: String,
    /// The database name, if known.
    pub database_name: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            provider: None,
            generated_at: now_rfc3339(),
            database_name: None,
        }
    }
}

/// Build an RFC 3339 timestamp from the current system time.
fn now_rfc3339() -> String {
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Format as ISO 8601 / RFC 3339 without external dependencies.
    // We compute year, month, day, hour, minute, second from Unix timestamp.
    let (y, month, day, h, min, s) = rfc3339_parts(secs);
    format!("{y:04}-{month:02}-{day:02}T{h:02}:{min:02}:{s:02}Z")
}

/// Convert a Unix timestamp (non-leap seconds since 1970-01-01) into
/// (year, month, day, hour, minute, second) in UTC.
fn rfc3339_parts(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    // Days since Unix epoch.
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;

    // Civil date from days since 1970-01-01 using a well-known algorithm.
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    (year, month, day, h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_epoch() {
        let (y, m, d, h, min, s) = rfc3339_parts(0);
        assert_eq!((y, m, d, h, min, s), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn rfc3339_midnight_2000() {
        // 2000-01-01T00:00:00Z = 946684800
        let (y, m, d, h, min, s) = rfc3339_parts(946_684_800);
        assert_eq!((y, m, d, h, min, s), (2000, 1, 1, 0, 0, 0));
    }

    #[test]
    fn rfc3339_leap_year() {
        // 2024-02-29T12:34:56Z = 1709210096
        let (y, m, d, h, min, s) = rfc3339_parts(1_709_210_096);
        assert_eq!((y, m, d, h, min, s), (2024, 2, 29, 12, 34, 56));
    }

    #[test]
    fn metadata_default_is_valid() {
        let m = Metadata::default();
        assert!(m.generated_at.ends_with('Z'));
        let len = m.generated_at.len();
        // "2026-07-14T01:23:45Z" = 20 chars
        assert!(len == 20, "expected 20 chars, got {len}: {}", m.generated_at);
        assert!(m.provider.is_none());
    }
}
