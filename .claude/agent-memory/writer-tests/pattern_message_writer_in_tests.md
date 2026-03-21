---
name: pattern_message_writer_in_tests
description: How to enqueue messages in Bevy 0.18 integration tests using a helper resource
type: project
---

# Enqueuing Messages in Bevy 0.18 Tests

## Pattern: helper resource + sender system

The standard pattern (from bump system tests — `apply_bump_velocity.rs` was deleted in refactor/unify-behaviors; see `behaviors/effects/speed_boost.rs` tests for current usage):

```rust
#[derive(Resource)]
struct TestMessage(Option<BumpPerformed>);

fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<BumpPerformed>) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<BumpPerformed>();
    app.add_systems(FixedUpdate, enqueue_from_resource.before(system_under_test));
    app
}
```

## Important: must call `app.add_message::<T>()`

Every message type used in a test app must be registered with `app.add_message::<T>()`.
For `AssetEvent<T>`, calling `app.init_asset::<T>()` automatically registers the message.

## Timing for FixedUpdate messages

Messages written in `FixedUpdate` are readable by systems in the SAME `FixedUpdate` step
(if they run after the writer). Use the `tick()` helper:

```rust
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}
```
