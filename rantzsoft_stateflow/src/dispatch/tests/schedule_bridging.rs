use super::helpers::*;

// --- Schedule placement: which dispatch schedule sees which writer schedule? ---
//
// Bevy 0.18 frame order:
//   First → PreUpdate → StateTransition → RunFixedMainLoop(
//       FixedFirst → FixedPreUpdate → FixedUpdate → FixedPostUpdate → FixedLast
//   ) → Update → PostUpdate → Last
//
// These tests write ChangeState from one schedule and place dispatch in another,
// proving which combinations work. A fire-once Local<bool> guard ensures the
// message is written exactly once.

// --- Writer in Update ---

#[test]
fn update_writer_update_dispatch() {
    assert!(
        probe_schedule_bridging(Update, Update),
        "Update → Update: message and dispatch in same schedule must work"
    );
}

#[test]
fn update_writer_post_update_dispatch() {
    assert!(
        probe_schedule_bridging(Update, PostUpdate),
        "Update → PostUpdate: dispatch after writer in same frame must work"
    );
}

#[test]
fn update_writer_fixed_post_update_dispatch() {
    // FixedPostUpdate runs BEFORE Update in the frame, so messages
    // written in Update won't be visible until the next frame's fixed loop.
    let result = probe_schedule_bridging(Update, FixedPostUpdate);
    // Record actual behavior — don't assert direction, just observe
    eprintln!("Update → FixedPostUpdate: {result}");
}

// --- Writer in FixedUpdate ---

#[test]
fn fixed_update_writer_update_dispatch() {
    // This is the broken case: FixedUpdate runs before Update, but
    // message double-buffering may hide the message.
    let result = probe_schedule_bridging(FixedUpdate, Update);
    eprintln!("FixedUpdate → Update: {result}");
}

#[test]
fn fixed_update_writer_post_update_dispatch() {
    let result = probe_schedule_bridging(FixedUpdate, PostUpdate);
    eprintln!("FixedUpdate → PostUpdate: {result}");
}

#[test]
fn fixed_update_writer_fixed_post_update_dispatch() {
    let result = probe_schedule_bridging(FixedUpdate, FixedPostUpdate);
    eprintln!("FixedUpdate → FixedPostUpdate: {result}");
}

/// Same as the FixedUpdate→FixedPostUpdate test but WITHOUT the `on_message`
/// run condition — dispatch runs unconditionally. Also verifies the fixed
/// tick actually runs by tracking a counter.
#[test]
fn fixed_update_writer_fixed_post_update_dispatch_no_run_condition() {
    #[derive(Resource, Default)]
    struct WriteCount(u32);

    fn write_once(
        mut writer: MessageWriter<ChangeState<TestState>>,
        mut count: ResMut<WriteCount>,
    ) {
        count.0 += 1;
        if count.0 == 1 {
            writer.write(ChangeState::new());
        }
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .init_resource::<WriteCount>()
        .add_message::<ChangeState<TestState>>()
        .add_message::<StateChanged<TestState>>()
        .init_resource::<RoutingTable<TestState>>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(20),
        ))
        .add_systems(FixedUpdate, write_once)
        .add_systems(FixedPostUpdate, dispatch_message_routes::<TestState>);

    app.world_mut()
        .resource_mut::<RoutingTable<TestState>>()
        .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
        .ok();

    for _ in 0..10 {
        app.update();
    }

    let writes = app.world().resource::<WriteCount>().0;
    let state = **app.world().resource::<State<TestState>>();
    eprintln!("FixedUpdate → FixedPostUpdate (no run_if): writes={writes} state={state:?}");
    assert!(
        writes > 0,
        "write_once must have fired at least once (verifies FixedUpdate ran)"
    );
    assert_eq!(
        state,
        TestState::AnimateIn,
        "dispatch in FixedPostUpdate should see FixedUpdate messages"
    );
}

// --- Writer in FixedPostUpdate ---

#[test]
fn fixed_post_update_writer_update_dispatch() {
    let result = probe_schedule_bridging(FixedPostUpdate, Update);
    eprintln!("FixedPostUpdate → Update: {result}");
}

#[test]
fn fixed_post_update_writer_post_update_dispatch() {
    let result = probe_schedule_bridging(FixedPostUpdate, PostUpdate);
    eprintln!("FixedPostUpdate → PostUpdate: {result}");
}
