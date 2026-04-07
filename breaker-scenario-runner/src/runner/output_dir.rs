//! Structured output directory and violation log file management.
//!
//! Creates per-run directories at `<BASE_DIR>/<YYYY-MM-DD>/<N>/` and writes
//! per-scenario violation log files within them. Also provides a `--clean`
//! function to remove all output.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::invariants::ViolationEntry;

/// Default base directory for scenario runner output.
pub const BASE_DIR: &str = "/tmp/breaker-scenario-runner";

/// Scans `<base_dir>/<date>/` for numeric subdirectories and returns max+1,
/// or 0 if none exist.
#[must_use]
pub fn next_run_number(base_dir: &Path, date: &str) -> u32 {
    let date_dir = base_dir.join(date);
    let Ok(entries) = fs::read_dir(&date_dir) else {
        return 0;
    };

    let mut max: Option<u32> = None;
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        if let Some(name) = entry.file_name().to_str()
            && let Ok(n) = name.parse::<u32>()
        {
            max = Some(max.map_or(n, |m: u32| m.max(n)));
        }
    }

    max.map_or(0, |m| m + 1)
}

/// Returns the current date as a `"YYYY-MM-DD"` string (UTC).
///
/// Uses `std::time::SystemTime` with Howard Hinnant's `civil_from_days`
/// algorithm to avoid external crate dependencies.
#[must_use]
pub fn today_date_string() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let days_since_epoch = i64::try_from(now.as_secs() / 86400).unwrap_or(0);
    let (year, month, day) = civil_from_days(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Howard Hinnant's `civil_from_days` algorithm. Converts a day count since
/// the Unix epoch (1970-01-01) into (year, month, day).
///
/// All intermediate conversions use `try_from` with safe defaults. The algorithm
/// guarantees values are in range for any date within +/- 5 million years.
fn civil_from_days(days_since_epoch: i64) -> (i32, i32, i32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u32::try_from(z - era * 146_097).unwrap_or(0);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = i64::from(yoe) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = i32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let m = i32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap_or(1);
    let y = i32::try_from(if m <= 2 { y + 1 } else { y }).unwrap_or(0);
    (y, m, d)
}

/// Creates `<base_dir>/<date>/<N>/` directory and returns the path.
///
/// # Errors
///
/// Returns an I/O error if directory creation fails.
pub fn create_run_dir(base_dir: &Path, date: &str) -> std::io::Result<PathBuf> {
    let n = next_run_number(base_dir, date);
    let path = base_dir.join(date).join(n.to_string());
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// Formats a single [`ViolationEntry`] as a human-readable log line.
///
/// With entity: `[frame N] Kind: message (entity: E)`
/// Without entity: `[frame N] Kind: message`
#[must_use]
pub fn format_violation_entry(entry: &ViolationEntry) -> String {
    entry.entity.map_or_else(
        || {
            format!(
                "[frame {}] {:?}: {}",
                entry.frame, entry.invariant, entry.message
            )
        },
        |entity| {
            format!(
                "[frame {}] {:?}: {} (entity: {entity:?})",
                entry.frame, entry.invariant, entry.message
            )
        },
    )
}

/// Writes a `violations.log` file under `<run_dir>/<scenario_name>/`.
///
/// No-op if `violations` is empty.
///
/// # Errors
///
/// Returns an I/O error if directory creation or file writing fails.
pub fn write_violations_log(
    run_dir: &Path,
    scenario_name: &str,
    violations: &[ViolationEntry],
) -> std::io::Result<()> {
    if violations.is_empty() {
        return Ok(());
    }

    let scenario_dir = run_dir.join(scenario_name);
    fs::create_dir_all(&scenario_dir)?;

    let log_path = scenario_dir.join("violations.log");
    let mut file = fs::File::create(log_path)?;

    for entry in violations {
        let line = format_violation_entry(entry);
        writeln!(file, "{line}")?;
    }

    Ok(())
}

/// Removes `base_dir` recursively. Returns `Ok(())` if it does not exist.
///
/// # Errors
///
/// Returns an I/O error if removal fails for a reason other than `NotFound`.
pub fn clean_output_dir(base_dir: &Path) -> std::io::Result<()> {
    match fs::remove_dir_all(base_dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
