use std::fmt::Write as _;

use crate::{
    invariants::ViolationEntry, log_capture::LogEntry, types::InvariantKind,
    verdict::ScenarioVerdict,
};

// ---------------------------------------------------------------------------
// Verbose output (--verbose flag)
// ---------------------------------------------------------------------------

pub(super) fn print_verbose_failures(
    scenario_name: &str,
    verdict: &ScenarioVerdict,
    violations: &[ViolationEntry],
    logs: &[LogEntry],
) {
    print!(
        "{}",
        format_verbose_failures(scenario_name, verdict, violations, logs)
    );
}

/// Returns verbose failure output as a `String` (no stdout printing).
pub(super) fn format_verbose_failures(
    scenario_name: &str,
    verdict: &ScenarioVerdict,
    violations: &[ViolationEntry],
    logs: &[LogEntry],
) -> String {
    let mut out = String::new();
    for reason in &verdict.reasons {
        let _ = writeln!(out, "  REASON [{scenario_name}]: {reason}");
    }
    for v in violations {
        let _ = writeln!(
            out,
            "  VIOLATION frame={} {:?} entity={:?}: {}",
            v.frame, v.invariant, v.entity, v.message
        );
        tracing::debug!(
            target: "breaker_scenario_runner",
            "violation frame={} invariant={:?} entity={:?}: {}",
            v.frame, v.invariant, v.entity, v.message
        );
    }
    for l in logs {
        let _ = writeln!(
            out,
            "  LOG frame={} {:?} target={}: {}",
            l.frame, l.level, l.target, l.message
        );
    }
    out
}

// ---------------------------------------------------------------------------
// Compact output (default)
// ---------------------------------------------------------------------------

pub(super) fn print_compact_failures(
    verdict: &ScenarioVerdict,
    violations: &[ViolationEntry],
    logs: &[LogEntry],
) {
    print!("{}", format_compact_failures(verdict, violations, logs));
}

/// Returns compact failure output as a `String` (no stdout printing).
pub(super) fn format_compact_failures(
    verdict: &ScenarioVerdict,
    violations: &[ViolationEntry],
    logs: &[LogEntry],
) -> String {
    let mut out = String::new();

    // Grouped violations.
    let violation_groups = group_violations(violations);
    for g in &violation_groups {
        let _ = writeln!(
            out,
            "  {:30} x{:<5} {}",
            format!("{:?}", g.invariant),
            g.count,
            format_frame_range(g.count, g.first_frame, g.last_frame)
        );
    }

    // Grouped logs.
    let log_groups = group_logs(logs);
    for g in &log_groups {
        let _ = writeln!(
            out,
            "  {:30} x{:<5} {}",
            format!("captured {:?} log", g.level),
            g.count,
            format_frame_range(g.count, g.first_frame, g.last_frame)
        );
        if g.count == 1 {
            let _ = writeln!(out, "    {}", g.message);
        }
    }

    // Health-check reasons (those not covered by violations or logs).
    for reason in &verdict.reasons {
        if is_health_check_reason(reason) {
            let _ = writeln!(out, "  {reason}");
        }
    }

    out
}

fn format_frame_range(count: u32, first: u32, last: u32) -> String {
    if count == 1 {
        format!("frame {first}")
    } else {
        format!("frames {first}..{last}")
    }
}

/// Returns `true` if the reason is a health-check (not a violation or log reason).
fn is_health_check_reason(reason: &str) -> bool {
    // Violation reasons come from InvariantKind::fail_reason() and log reasons
    // start with "captured". Health checks are everything else.
    !reason.starts_with("captured ") && !is_invariant_fail_reason(reason)
}

/// Returns `true` if the reason matches any `InvariantKind::fail_reason()`.
pub(super) fn is_invariant_fail_reason(reason: &str) -> bool {
    InvariantKind::ALL.iter().any(|v| v.fail_reason() == reason)
}

// ---------------------------------------------------------------------------
// Grouping types and functions
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub(super) struct ViolationGroup {
    pub(super) invariant:   InvariantKind,
    pub(super) count:       u32,
    pub(super) first_frame: u32,
    pub(super) last_frame:  u32,
}

#[derive(Debug, PartialEq)]
pub(super) struct LogGroup {
    pub(super) level:       bevy::log::Level,
    pub(super) message:     String,
    pub(super) count:       u32,
    pub(super) first_frame: u32,
    pub(super) last_frame:  u32,
}

pub(super) fn group_violations(violations: &[ViolationEntry]) -> Vec<ViolationGroup> {
    use std::collections::HashMap;

    let mut map: HashMap<InvariantKind, (u32, u32, u32)> = HashMap::new();
    let mut insertion_order: Vec<InvariantKind> = Vec::new();

    for v in violations {
        let entry = map.entry(v.invariant).or_insert_with(|| {
            insertion_order.push(v.invariant);
            (0, v.frame, v.frame)
        });
        entry.0 += 1;
        entry.1 = entry.1.min(v.frame);
        entry.2 = entry.2.max(v.frame);
    }

    insertion_order
        .into_iter()
        .filter_map(|kind| {
            map.get(&kind).map(|&(count, first, last)| ViolationGroup {
                invariant: kind,
                count,
                first_frame: first,
                last_frame: last,
            })
        })
        .collect()
}

pub(super) fn group_logs(logs: &[LogEntry]) -> Vec<LogGroup> {
    use std::collections::HashMap;

    type Key = (bevy::log::Level, String);
    let mut map: HashMap<Key, (u32, u32, u32)> = HashMap::new();
    let mut insertion_order: Vec<Key> = Vec::new();

    for l in logs {
        let key: Key = (l.level, l.message.clone());
        let entry = map.entry(key.clone()).or_insert_with(|| {
            insertion_order.push(key);
            (0, l.frame, l.frame)
        });
        entry.0 += 1;
        entry.1 = entry.1.min(l.frame);
        entry.2 = entry.2.max(l.frame);
    }

    insertion_order
        .into_iter()
        .filter_map(|key| {
            map.get(&key).map(|&(count, first, last)| LogGroup {
                level: key.0,
                message: key.1,
                count,
                first_frame: first,
                last_frame: last,
            })
        })
        .collect()
}
