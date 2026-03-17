---
name: Message Capture Pattern
description: How to capture Bevy messages (MessageReader<T>) into a Resource for assertion in integration tests
type: reference
---

Use a `Resource` accumulator with a system that drains the reader each tick:

```rust
#[derive(Resource, Default)]
struct CapturedMessages(Vec<MyMessage>);

fn collect_messages(mut reader: MessageReader<MyMessage>, mut captured: ResMut<CapturedMessages>) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}
```

Register the resource and the collector system in `test_app()`:

```rust
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.register_message::<MyMessage>();
    app.init_resource::<CapturedMessages>();
    app.add_systems(Update, collect_messages);
    app
}
```

After `tick(&mut app)`, assert on `app.world().resource::<CapturedMessages>().0`.
