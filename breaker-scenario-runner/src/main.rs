//! Scenario runner — automated gameplay testing tool.
//!
//! Run a scenario file to validate gameplay invariants over N frames.
//!
//! Usage:
//!   `cargo dscenario -- -s aegis_chaos`
//!   `cargo dscenario -- --headless -s aegis_chaos`
//!   `cargo dscenario -- --all --headless`

fn main() {
    breaker_scenario_runner::run();
}
