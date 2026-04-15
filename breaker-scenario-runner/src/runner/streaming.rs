//! Count-based streaming pool for managing concurrent scenario execution.

use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use super::{
    execution::{ChildResult, SubprocessSpec},
    tiling,
};

/// A reusable free-list of visual tile slots.
///
/// Visual mode tiles child windows into a grid sized to the concurrency
/// cap — not the total job count — so the screen stays fully populated
/// even when running more scenarios than parallel workers. Each spawn
/// acquires a slot; each completion releases it back for the next job.
#[derive(Debug)]
pub struct SlotPool {
    free:     Vec<usize>,
    capacity: usize,
}

impl SlotPool {
    /// Creates a pool with `capacity` slots, all initially free.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        // Reverse so `pop()` yields 0, 1, 2, ... in order.
        let free = (0..capacity).rev().collect();
        Self { free, capacity }
    }

    /// Returns the total slot capacity (never changes after construction).
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Pops the next free slot, or `None` if exhausted.
    pub fn acquire(&mut self) -> Option<usize> {
        self.free.pop()
    }

    /// Returns `slot` to the pool so a later `acquire` can reuse it.
    ///
    /// In debug builds, panics if `slot` is out of range or already free.
    pub fn release(&mut self, slot: usize) {
        debug_assert!(
            slot < self.capacity,
            "slot {slot} out of range for SlotPool capacity {}",
            self.capacity
        );
        debug_assert!(
            !self.free.contains(&slot),
            "slot {slot} released twice (double-free)"
        );
        self.free.push(slot);
    }
}

/// A pure state machine that tracks how many items can run concurrently,
/// how many have been dispatched, and how many have completed.
#[derive(Debug)]
pub struct StreamingPool {
    max_concurrent:  usize,
    total:           usize,
    next_index:      usize,
    active_count:    usize,
    completed_count: usize,
}

impl StreamingPool {
    /// Creates a new pool. `max_concurrent` is clamped to at least 1.
    #[must_use]
    pub fn new(max_concurrent: usize, total: usize) -> Self {
        Self {
            max_concurrent: max_concurrent.max(1),
            total,
            next_index: 0,
            active_count: 0,
            completed_count: 0,
        }
    }

    /// Returns `true` if there is capacity to start another item.
    #[must_use]
    pub const fn can_start(&self) -> bool {
        self.active_count < self.max_concurrent && self.next_index < self.total
    }

    /// Starts the next item and returns its index.
    pub fn start_next(&mut self) -> usize {
        debug_assert!(self.can_start());
        let index = self.next_index;
        self.next_index += 1;
        self.active_count += 1;
        index
    }

    /// Marks one active item as complete.
    pub fn mark_complete(&mut self) {
        debug_assert!(self.active_count > 0);
        self.active_count -= 1;
        self.completed_count += 1;
    }

    /// Returns `true` if all items have been completed.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.completed_count == self.total
    }

    /// Returns the number of currently active (started but not completed) items.
    #[must_use]
    pub const fn active_count(&self) -> usize {
        self.active_count
    }

    /// Returns the number of completed items.
    #[must_use]
    pub const fn completed_count(&self) -> usize {
        self.completed_count
    }

    /// Returns the number of items not yet dispatched.
    #[must_use]
    pub const fn remaining_count(&self) -> usize {
        self.total - self.next_index
    }
}

/// Spawns subprocesses using a streaming pool for concurrent execution.
///
/// Continuously fills available worker slots as children complete, so
/// utilisation stays high. In visual mode, tile slots are bounded by
/// `max_concurrent` (not `specs.len()`) and are recycled by [`SlotPool`]
/// so the screen stays fully populated while jobs hand off.
pub(super) fn spawn_streaming(
    specs: &[SubprocessSpec],
    visual: bool,
    verbose: bool,
    max_concurrent: usize,
) -> Result<Vec<ChildResult>, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to determine current executable path: {e}"))?;

    let mut pool = StreamingPool::new(max_concurrent, specs.len());
    let mut results: Vec<Option<ChildResult>> = (0..specs.len()).map(|_| None).collect();
    let mut active: Vec<(usize, Option<usize>, std::process::Child)> = Vec::new();

    // Visual tile grid is sized to the concurrency cap, not the total job
    // count — so slots stay fully populated as workers hand off jobs.
    let tile_count = max_concurrent.max(1).min(specs.len());
    let mut slot_pool = visual.then(|| SlotPool::new(tile_count));

    while !pool.is_done() {
        // Spawn phase — fill available slots.
        while pool.can_start() {
            let idx = pool.start_next();
            let spec = &specs[idx];

            let mut cmd = Command::new(&exe);
            for arg in &spec.extra_args {
                cmd.arg(arg);
            }
            if visual {
                cmd.arg("--visual");
            }
            if verbose {
                cmd.arg("-v");
            }
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            let slot = slot_pool.as_mut().and_then(SlotPool::acquire);
            if let Some(s) = slot {
                for (key, value) in tiling::tile_config_env_vars(s, tile_count) {
                    cmd.env(key, value);
                }
            }

            match cmd.spawn() {
                Ok(child) => active.push((idx, slot, child)),
                Err(e) => {
                    if let (Some(s), Some(sp)) = (slot, slot_pool.as_mut()) {
                        sp.release(s);
                    }
                    eprintln!(
                        "Failed to spawn subprocess for [{}]: {e}",
                        spec.display_name
                    );
                    results[idx] = Some(ChildResult {
                        name:   spec.display_name.clone(),
                        passed: false,
                        stdout: String::new(),
                        stderr: format!("spawn error: {e}"),
                    });
                    pool.mark_complete();
                }
            }
        }

        let any_finished = poll_active_children(
            &mut active,
            specs,
            &mut results,
            &mut pool,
            slot_pool.as_mut(),
        );

        // Sleep phase — avoid busy-waiting if no child finished this iteration.
        if !any_finished && !pool.is_done() {
            thread::sleep(Duration::from_millis(1));
        }
    }

    Ok(results.into_iter().flatten().collect())
}

/// Polls active child processes for completion, collecting results.
///
/// Iterates backwards for safe `swap_remove`. Returns `true` if any child
/// finished during this poll cycle. When a child finishes, its visual tile
/// slot (if any) is released back to `slot_pool` so the next spawn reuses it.
fn poll_active_children(
    active: &mut Vec<(usize, Option<usize>, std::process::Child)>,
    specs: &[SubprocessSpec],
    results: &mut [Option<ChildResult>],
    pool: &mut StreamingPool,
    mut slot_pool: Option<&mut SlotPool>,
) -> bool {
    let mut any_finished = false;
    let mut i = active.len();
    while i > 0 {
        i -= 1;
        match active[i].2.try_wait() {
            Ok(Some(_status)) => {
                let (idx, slot, child) = active.swap_remove(i);
                match child.wait_with_output() {
                    Ok(output) => {
                        let stdout = String::from_utf8(output.stdout)
                            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
                        let stderr = String::from_utf8(output.stderr)
                            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
                        results[idx] = Some(ChildResult {
                            name: specs[idx].display_name.clone(),
                            passed: output.status.success(),
                            stdout,
                            stderr,
                        });
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to wait on child process [{}]: {e}",
                            specs[idx].display_name
                        );
                        results[idx] = Some(ChildResult {
                            name:   specs[idx].display_name.clone(),
                            passed: false,
                            stdout: String::new(),
                            stderr: format!("wait error: {e}"),
                        });
                    }
                }
                if let (Some(s), Some(sp)) = (slot, slot_pool.as_deref_mut()) {
                    sp.release(s);
                }
                pool.mark_complete();
                any_finished = true;
            }
            Ok(None) => {
                // Still running, skip.
            }
            Err(e) => {
                let (idx, slot, _child) = active.swap_remove(i);
                eprintln!(
                    "Failed to check child process [{}]: {e}",
                    specs[idx].display_name
                );
                results[idx] = Some(ChildResult {
                    name:   specs[idx].display_name.clone(),
                    passed: false,
                    stdout: String::new(),
                    stderr: format!("try_wait error: {e}"),
                });
                if let (Some(s), Some(sp)) = (slot, slot_pool.as_deref_mut()) {
                    sp.release(s);
                }
                pool.mark_complete();
                any_finished = true;
            }
        }
    }
    any_finished
}
