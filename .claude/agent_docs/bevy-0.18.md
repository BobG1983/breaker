# Bevy 0.18 Patterns — IMPORTANT

Claude MUST use Bevy 0.18 APIs. Key differences from older tutorials:

- **Messages, not Events**: Use `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`, `app.add_message::<T>()`. The `Event` trait is ONLY for observable/triggered events.
- **Required Components, not Bundles**: `SpriteBundle`, `NodeBundle`, etc. are DEPRECATED. Use `#[require(...)]` attribute on components.
- **Spawn with tuples**: `commands.spawn((ComponentA, ComponentB))` — no bundles.
- **Camera**: `commands.spawn(Camera2d)` — no bundle.
- **Sprites**: `Sprite::from_image(handle)` or `Sprite::from_atlas_image(handle, atlas)`.
- **Feature profiles**: Use `features = ["2d"]` not the full default feature set.
