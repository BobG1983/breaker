//! Scenario app construction, run orchestration, and post-run evaluation.

mod build;
mod evaluate;
mod run;
mod types;
mod window;

pub(crate) use run::run_scenario;
#[cfg(test)]
pub(crate) use {
    evaluate::{
        collect_and_evaluate, drain_remaining_logs, guarded_update, is_timed_out, should_fail_fast,
    },
    types::{EvalSnapshot, SharedEvalBuffer, snapshot_eval_data},
    window::{apply_tile_layout, sync_ui_scale},
};
