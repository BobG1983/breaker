---
name: pattern_asset_event_testing
description: How to trigger and test AssetEvent<T>::Modified in Bevy 0.18 integration tests
type: project
---

# Testing AssetEvent<T> in Bevy 0.18

## Key facts

- `AssetEvent<A>` is a `Message` in Bevy 0.18 (NOT an Event). Use `MessageReader<AssetEvent<T>>`.
- `app.init_asset::<T>()` registers both the `Assets<T>` resource AND `add_message::<AssetEvent<T>>()`.
- `AssetPlugin::default()` must be added for `init_asset` to work.
- `asset_events` system runs in `PostUpdate` — flushes queued events to the message buffer.
- `message_update_system` runs in `First` — rotates the message buffer (old writes become readable).

## Timing model for Modified events

```
Frame N:   assets.get_mut(id)      ← queues Modified in Assets.queued_events
  PostUpdate: asset_events          ← writes AssetEvent::Modified to message buffer
Frame N+1: First: message_update   ← rotates buffer, Modified is now readable
           Update: system reads it  ← react to Modified
```

Therefore, after calling `assets.get_mut(id)`, need **2 `app.update()` calls** before the system sees Modified.

## Test template for "config updated on Modified"

```rust
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<BoltDefaults>();
    app.init_resource::<BoltConfig>();
    app.add_systems(Update, propagate_bolt_defaults);
    app
}

// In test:
let handle = {
    let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
    assets.add(defaults)
};
app.world_mut().insert_resource(make_collection(handle.clone()));

// Let Added event settle (2 updates)
app.update();
app.update();

// Trigger Modified
{
    let mut assets = app.world_mut().resource_mut::<Assets<BoltDefaults>>();
    let asset = assets.get_mut(handle.id()).expect("asset should exist");
    asset.some_field = new_value;
}

// Let Modified event propagate (2 updates)
app.update();
app.update();

let config = app.world().resource::<BoltConfig>();
assert_eq!(config.some_field, new_value);
```

## "Config unchanged" negative test pattern

For testing that no-event = no-change:
- Insert asset → 2 updates (only Added fires)
- Assert config unchanged

For testing wrong-handle = no-change:
- Insert registered handle + unregistered handle
- Mutate unregistered handle
- 2 updates
- Assert config unchanged

Both pass even on stubs (because stubs do nothing). This is intentional — negative tests should pass in both stub state and implemented state.

## DefaultsCollection construction in tests

```rust
fn make_collection(bolt: Handle<BoltDefaults>) -> DefaultsCollection {
    DefaultsCollection {
        bolt,
        breaker: Handle::default(),
        cell_defaults: Handle::default(),
        playfield: Handle::default(),
        input: Handle::default(),
        mainmenu: Handle::default(),
        timerui: Handle::default(),
        chipselect: Handle::default(),
        cells: vec![],
        nodes: vec![],
        breakers: vec![],
        chips: vec![],
        difficulty: Handle::default(),
    }
}
```
