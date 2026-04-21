//! Pre-gate message drain harness.
//!
//! Shared across any domain whose systems are scheduled under a
//! `.run_if(...)` gate that can open mid-session while a `MessageReader`
//! has unread buffered events. Exercises the canonical
//! "write-while-gated-off → flip gate on → tick → assert no side effect"
//! sequence so every gated reader system can prove it drains via
//! `reader.clear()` in its in-body gate-check path.

use bevy::prelude::*;

use super::tick_helper::tick;

/// Pre-gate drain harness. Executes the canonical "write-while-gated-off
/// → flip gate on → tick → assert no side effect" sequence.
///
/// Flow:
/// 1. `build_app()` constructs an `App` with systems registered and all
///    resources initialised, but WITHOUT the protocol being active.
/// 2. `write_messages(&mut app)` writes one or more `M` messages while
///    the gate is off (protocol not in `ActiveProtocols`).
/// 3. `tick(&mut app)` ticks once. The in-body drain path must
///    `reader.clear()` the buffered message.
/// 4. `open_gate(&mut app)` flips the gate on (inserts the protocol
///    into `ActiveProtocols`, or otherwise enables the run-if).
/// 5. `tick(&mut app)` ticks once more. The system now runs but its
///    `MessageReader` cursor was advanced on the previous tick via
///    `reader.clear()` in the in-body gate-check path, so the stale
///    message MUST NOT produce a side effect.
/// 6. `assert_no_side_effect(&app)` — the caller asserts (via Bevy
///    assertions, panics, etc.) that no downstream state changed.
///
/// The helper does not itself assert anything — it orchestrates the
/// ticks. The caller's `assert_no_side_effect` closure is where
/// failures materialise.
///
/// `tick()` (not `App::update()`) is required: test fixtures run under
/// `TimeUpdateStrategy::ManualDuration(Duration::ZERO)`, and only
/// `tick()` advances `Time<Virtual>` enough for `FixedUpdate` to fire.
pub(crate) fn assert_pregate_drain<BuildApp, WriteMessages, OpenGate, AssertNoSideEffect>(
    build_app: BuildApp,
    write_messages: WriteMessages,
    open_gate: OpenGate,
    assert_no_side_effect: AssertNoSideEffect,
) where
    BuildApp: FnOnce() -> App,
    WriteMessages: FnOnce(&mut App),
    OpenGate: FnOnce(&mut App),
    AssertNoSideEffect: FnOnce(&App),
{
    let mut app = build_app();
    write_messages(&mut app);
    tick(&mut app); // tick 1: gate off — in-body drain must `reader.clear()`
    open_gate(&mut app);
    tick(&mut app); // tick 2: gate on — reader already drained → no replay
    assert_no_side_effect(&app);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{ecs::message::Messages, prelude::*, time::TimeUpdateStrategy};

    use super::assert_pregate_drain;

    // ── Fixture types (test-only, local to this module) ─────────────────

    #[derive(Message, Clone)]
    struct TestPing;

    #[derive(Resource, Default)]
    struct FixtureCounter(u32);

    #[derive(Resource, Default)]
    struct FixtureGate(bool);

    // ── Fixture systems ─────────────────────────────────────────────────

    /// Broken variant — reads the gate resource but does NOT check it
    /// in-body. Reproduces the latent retroactive-consume bug.
    fn broken_reader(
        mut reader: MessageReader<TestPing>,
        mut counter: ResMut<FixtureCounter>,
        _gate: Res<FixtureGate>,
    ) {
        // NO in-body gate check — this reproduces the bug.
        for _ in reader.read() {
            counter.0 += 1;
        }
    }

    /// Fixed variant — checks the gate in-body and drains via
    /// `reader.clear()` when gate is off. Parallels the Burnout retrofit.
    fn fixed_reader(
        mut reader: MessageReader<TestPing>,
        mut counter: ResMut<FixtureCounter>,
        gate: Res<FixtureGate>,
    ) {
        if !gate.0 {
            reader.clear();
            return;
        }
        for _ in reader.read() {
            counter.0 += 1;
        }
    }

    // ── App builders ────────────────────────────────────────────────────

    fn build_app_with_broken_reader() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.add_message::<TestPing>();
        app.init_resource::<FixtureCounter>();
        app.init_resource::<FixtureGate>();
        // MUST be FixedUpdate (not Update) — matches the schedule the
        // retrofitted production Burnout systems run in and makes the
        // `tick()` helper's fixed-timestep advance meaningful. A fixture
        // registered in `Update` would make the self-test vacuous.
        app.add_systems(
            FixedUpdate,
            broken_reader.run_if(|gate: Res<FixtureGate>| gate.0),
        );
        app
    }

    fn build_app_with_fixed_reader() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.add_message::<TestPing>();
        app.init_resource::<FixtureCounter>();
        app.init_resource::<FixtureGate>();
        // NO plugin-level `.run_if` — this fixture mirrors the production
        // retrofit pattern where the `.run_if` gate is dropped in favor of
        // the in-body gate check. `fixed_reader` self-gates and drains via
        // `reader.clear()` when the gate is off, so the buffered message
        // is discarded on the gate-off tick before the gate opens.
        app.add_systems(FixedUpdate, fixed_reader);
        app
    }

    // ── Closure factories ───────────────────────────────────────────────

    fn write_one_ping(app: &mut App) {
        app.world_mut()
            .resource_mut::<Messages<TestPing>>()
            .write(TestPing);
    }

    fn open_fixture_gate(app: &mut App) {
        app.world_mut().resource_mut::<FixtureGate>().0 = true;
    }

    // ── Behavior 3 — helper panics on BROKEN fixture ────────────────────

    #[test]
    #[should_panic(expected = "retroactively consumed")]
    fn helper_panics_when_applied_to_broken_fixture_system() {
        assert_pregate_drain(
            build_app_with_broken_reader,
            write_one_ping,
            open_fixture_gate,
            |app| {
                // Stable substring `retroactively consumed` — MUST NOT be
                // reworded. The `#[should_panic(expected = ...)]`
                // attribute above matches on this substring.
                let count = app.world().resource::<FixtureCounter>().0;
                assert_eq!(
                    count, 0,
                    "broken_reader retroactively consumed {count} buffered messages",
                );
            },
        );
    }

    // ── Behavior 4 — helper passes on FIXED fixture ─────────────────────

    #[test]
    fn helper_passes_when_applied_to_fixed_fixture_system() {
        assert_pregate_drain(
            build_app_with_fixed_reader,
            write_one_ping,
            open_fixture_gate,
            |app| {
                let count = app.world().resource::<FixtureCounter>().0;
                assert_eq!(
                    count, 0,
                    "fixed_reader must NOT observe buffered ping — but observed {count}",
                );
            },
        );
    }

    // ── Behavior 5 — helper drains all messages, not just the first ─────

    /// Regression guard: a partial-drain implementation that only advances
    /// past the first message would pass the single-ping self-tests. This
    /// test writes THREE pings while the gate is off and asserts the fixed
    /// reader observes zero of them after the gate opens — only a full
    /// `reader.clear()` (not a skip-first implementation) can satisfy.
    #[test]
    fn helper_drains_all_buffered_messages_on_fixed_fixture() {
        fn write_three_pings(app: &mut App) {
            let mut messages = app.world_mut().resource_mut::<Messages<TestPing>>();
            messages.write(TestPing);
            messages.write(TestPing);
            messages.write(TestPing);
        }

        assert_pregate_drain(
            build_app_with_fixed_reader,
            write_three_pings,
            open_fixture_gate,
            |app| {
                let count = app.world().resource::<FixtureCounter>().0;
                assert_eq!(
                    count, 0,
                    "fixed_reader must drain ALL buffered pings — observed {count}",
                );
            },
        );
    }
}
