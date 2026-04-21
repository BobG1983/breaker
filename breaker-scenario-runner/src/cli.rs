//! CLI argument parsing for the scenario runner binary.
//!
//! Pure helpers — no runtime dependencies. Contains the clap [`Args`] struct,
//! the [`FailFastMode`] / [`ExecutionMode`] flag groups, and the pure functions
//! [`resolve_fail_fast`] and [`parse_loop_count`] that `main.rs` uses.

use clap::Parser;

/// Automated gameplay scenario runner.
#[derive(Parser)]
#[command(about = "Automated gameplay scenario runner")]
pub struct Args {
    /// Scenario name to run (stem of a `.scenario.ron` file in `scenarios/`)
    #[arg(short = 's', long)]
    pub scenario: Option<String>,

    /// Run all scenarios in the `scenarios/` directory tree
    #[arg(long)]
    pub all: bool,

    /// Run with a window for visual debugging
    #[arg(long)]
    pub visual: bool,

    /// Print all violations and logs verbatim (default: grouped compact output)
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Fail-fast behavior flags (`--fail-fast` / `--no-fail-fast`).
    #[command(flatten)]
    pub fail_fast_mode: FailFastMode,

    /// Execution mode flags (`--parallel` / `--serial`).
    #[command(flatten)]
    pub execution: ExecutionMode,

    /// Repeat the entire run N times
    #[arg(short = 'l', long = "loop", value_parser = parse_loop_count)]
    pub loops: Option<usize>,
}

/// Fail-fast mode: `--fail-fast` or `--no-fail-fast` (clap enforces mutual exclusion via `overrides_with`).
#[derive(clap::Args)]
pub struct FailFastMode {
    /// Stop scenario on first invariant violation (default: on for `--all`, off for `-s`)
    #[arg(long, overrides_with = "no_fail_fast")]
    pub fail_fast: bool,

    /// Run scenarios to completion even with violations (overrides default)
    #[arg(long, overrides_with = "fail_fast")]
    pub no_fail_fast: bool,
}

/// Execution mode: `--parallel` or `--serial` (clap enforces mutual exclusion).
#[derive(clap::Args)]
pub struct ExecutionMode {
    /// Max parallel subprocesses: a number or "all" (default: 32)
    #[arg(short = 'p', long, conflicts_with = "serial")]
    pub parallel: Option<String>,

    /// Run in-process sequentially, no subprocesses
    #[arg(long, conflicts_with = "parallel")]
    pub serial: bool,

    /// Internal: marks this process as a stress-copy subprocess.
    /// Skips stress expansion to prevent infinite recursion.
    #[arg(long, hide = true)]
    pub stress_copy: bool,
}

/// Resolves the effective `fail_fast` value from CLI flags and run mode.
///
/// Resolution order:
/// 1. `--fail-fast` explicitly present → `true`
/// 2. `--no-fail-fast` explicitly present → `false`
/// 3. Neither present → default to `all` (on for `--all`, off for `-s`)
#[must_use]
pub const fn resolve_fail_fast(fail_fast: bool, no_fail_fast: bool, all: bool) -> bool {
    if fail_fast {
        true
    } else if no_fail_fast {
        false
    } else {
        all
    }
}

/// Parses a loop count string into a positive `usize`.
///
/// # Errors
/// Returns `Err` if `s` is not a valid integer or if the parsed value is zero.
pub fn parse_loop_count(s: &str) -> Result<usize, String> {
    let n: usize = s
        .parse()
        .map_err(|_| format!("invalid loop count: \"{s}\""))?;
    if n == 0 {
        return Err("--loop must be a positive number".to_owned());
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_loop_count_accepts_positive_number() {
        assert_eq!(parse_loop_count("5"), Ok(5));
    }

    #[test]
    fn parse_loop_count_rejects_zero() {
        let result = parse_loop_count("0");
        assert!(result.is_err(), "expected error for 0, got: {result:?}");
    }

    #[test]
    fn parse_loop_count_rejects_non_numeric() {
        let result = parse_loop_count("abc");
        assert!(result.is_err(), "expected error for 'abc', got: {result:?}");
    }

    #[test]
    fn stress_copy_flag_parses() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--stress-copy"]);
        assert!(args.execution.stress_copy, "stress_copy must be true");
        assert_eq!(args.scenario.as_deref(), Some("foo"));
    }

    #[test]
    fn stress_copy_flag_defaults_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo"]);
        assert!(
            !args.execution.stress_copy,
            "stress_copy must default to false"
        );
    }

    // -----------------------------------------------------------------
    // --fail-fast / --no-fail-fast flags: parsing and resolution
    // -----------------------------------------------------------------

    // Behavior 1: --fail-fast flag sets fail_fast field to true
    #[test]
    fn fail_fast_flag_sets_fail_fast_field_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast"]);
        assert!(
            args.fail_fast_mode.fail_fast,
            "fail_fast must be true when --fail-fast is provided"
        );
        assert!(
            !args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be false when only --fail-fast is provided"
        );
        assert_eq!(args.scenario.as_deref(), Some("foo"));
    }

    // Behavior 1 edge: --fail-fast combined with --all
    #[test]
    fn fail_fast_flag_with_all_sets_both_fields() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--fail-fast"]);
        assert!(args.fail_fast_mode.fail_fast, "fail_fast must be true");
        assert!(args.all, "all must be true");
    }

    // Behavior 2: --no-fail-fast flag sets no_fail_fast field to true
    #[test]
    fn no_fail_fast_flag_sets_no_fail_fast_field_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--no-fail-fast"]);
        assert!(
            args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be true when --no-fail-fast is provided"
        );
        assert!(
            !args.fail_fast_mode.fail_fast,
            "fail_fast must be false when only --no-fail-fast is provided"
        );
        assert!(args.all, "all must be true");
    }

    // Behavior 2 edge: --no-fail-fast combined with -s
    #[test]
    fn no_fail_fast_flag_with_scenario_sets_no_fail_fast_field() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--no-fail-fast"]);
        assert!(
            args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be true"
        );
        assert_eq!(args.scenario.as_deref(), Some("foo"));
    }

    // Behavior 3: neither flag + --all resolves to true
    #[test]
    fn neither_flag_with_all_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when neither flag given with --all"
        );
    }

    // Behavior 3 edge: --all with --serial and no fail-fast flags
    #[test]
    fn neither_flag_with_all_and_serial_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--serial"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when --all --serial given without fail-fast flags"
        );
    }

    // Behavior 4: neither flag + -s resolves to false
    #[test]
    fn neither_flag_with_scenario_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when neither flag given with -s"
        );
    }

    // Behavior 4 edge: -s with --verbose and no fail-fast flags
    #[test]
    fn neither_flag_with_scenario_and_verbose_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "-v"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when -s -v given without fail-fast flags"
        );
    }

    // Behavior 5: --fail-fast with -s resolves to true (explicit override)
    #[test]
    fn explicit_fail_fast_with_scenario_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when --fail-fast explicitly given with -s"
        );
    }

    // Behavior 5 edge: reversed order
    #[test]
    fn explicit_fail_fast_before_scenario_resolves_to_true() {
        let args = Args::parse_from(["breaker_scenario_runner", "--fail-fast", "-s", "foo"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true regardless of flag order"
        );
    }

    // Behavior 6: --no-fail-fast with --all resolves to false (explicit override)
    #[test]
    fn explicit_no_fail_fast_with_all_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "--all", "--no-fail-fast"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when --no-fail-fast explicitly given with --all"
        );
    }

    // Behavior 6 edge: reversed order
    #[test]
    fn explicit_no_fail_fast_before_all_resolves_to_false() {
        let args = Args::parse_from(["breaker_scenario_runner", "--no-fail-fast", "--all"]);
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false regardless of flag order"
        );
    }

    // Behavior 7: --fail-fast=true syntax is rejected by clap
    #[test]
    fn fail_fast_equals_value_syntax_is_rejected() {
        let result =
            Args::try_parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast=true"]);
        assert!(
            result.is_err(),
            "clap should reject --fail-fast=true for boolean flags"
        );
    }

    // Behavior 7 edge: --fail-fast=false is also rejected
    #[test]
    fn fail_fast_equals_false_syntax_is_rejected() {
        let result =
            Args::try_parse_from(["breaker_scenario_runner", "-s", "foo", "--fail-fast=false"]);
        assert!(
            result.is_err(),
            "clap should reject --fail-fast=false for boolean flags"
        );
    }

    // Behavior 7 edge: --no-fail-fast=true is also rejected
    #[test]
    fn no_fail_fast_equals_true_syntax_is_rejected() {
        let result = Args::try_parse_from([
            "breaker_scenario_runner",
            "-s",
            "foo",
            "--no-fail-fast=true",
        ]);
        assert!(
            result.is_err(),
            "clap should reject --no-fail-fast=true for boolean flags"
        );
    }

    // Behavior 8: both flags given -- last one wins via clap overrides_with
    #[test]
    fn both_flags_given_last_wins_no_fail_fast_last() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "--all",
            "--fail-fast",
            "--no-fail-fast",
        ]);
        assert!(
            !args.fail_fast_mode.fail_fast,
            "fail_fast must be false when --no-fail-fast is last (overrides_with)"
        );
        assert!(
            args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be true when --no-fail-fast is last"
        );
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            !resolved,
            "resolved fail_fast must be false when --no-fail-fast wins"
        );
    }

    // Behavior 8 edge: reversed order -- --fail-fast last
    #[test]
    fn both_flags_given_last_wins_fail_fast_last() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "--all",
            "--no-fail-fast",
            "--fail-fast",
        ]);
        assert!(
            args.fail_fast_mode.fail_fast,
            "fail_fast must be true when --fail-fast is last (overrides_with)"
        );
        assert!(
            !args.fail_fast_mode.no_fail_fast,
            "no_fail_fast must be false when --fail-fast is last"
        );
        let resolved = resolve_fail_fast(
            args.fail_fast_mode.fail_fast,
            args.fail_fast_mode.no_fail_fast,
            args.all,
        );
        assert!(
            resolved,
            "resolved fail_fast must be true when --fail-fast wins"
        );
    }

    // Behavior 9: --fail-fast combines with other flags without conflict
    #[test]
    fn fail_fast_combines_with_all_verbose_serial() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "--all",
            "--fail-fast",
            "-v",
            "--serial",
        ]);
        assert!(args.fail_fast_mode.fail_fast, "fail_fast must be true");
        assert!(args.all, "all must be true");
        assert!(args.verbose, "verbose must be true");
        assert!(args.execution.serial, "serial must be true");
    }

    // Behavior 9 edge: --fail-fast with --stress-copy
    #[test]
    fn fail_fast_combines_with_stress_copy() {
        let args = Args::parse_from([
            "breaker_scenario_runner",
            "-s",
            "foo",
            "--fail-fast",
            "--stress-copy",
        ]);
        assert!(args.fail_fast_mode.fail_fast, "fail_fast must be true");
        assert!(args.execution.stress_copy, "stress_copy must be true");
    }
}
