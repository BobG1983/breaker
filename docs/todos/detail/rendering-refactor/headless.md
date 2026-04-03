# Headless Mode

How `rantzsoft_vfx` operates without rendering — for the scenario runner and tests.

## Approach: Plugin Headless Mode

`RantzVfxPlugin::headless()` registers:
- All message types (so game code can send messages without compile errors)
- Completion message senders (so `TransitionComplete` still fires for game logic that depends on it)
- Recipe loading pipeline (so `RecipeStore` is populated and recipe names can be validated)

Does NOT register:
- Rendering systems (mesh generation, shader updates, material creation)
- Post-processing pipeline
- Particle system
- Screen effect handlers (shake, distortion, etc.)
- Modifier computation systems

## Behavior

- `AttachVisuals` messages are received but ignored — no mesh/material attached
- `ExecuteRecipe` messages are received but no VFX entities spawned
- `SetModifier`/`AddModifier`/`RemoveModifier` messages are received but no visual state computed
- `TransitionComplete` is NOT owned by the crate — it's a `screen/` domain message. In headless mode, `screen/transition/` handles it directly (fires immediately with no animation delay). The crate has no knowledge of transitions.
- Screen effect messages are dropped silently

## Why Not "No Plugin At All"

If the plugin isn't registered, game code that sends VFX messages would need `#[cfg]` guards everywhere or the messages would be unregistered (causing panics or silent failures depending on Bevy version). Headless mode avoids this — the message types exist, they just don't do visual work.

## Integration with Existing Headless

The game currently has `Game::headless()` which skips `RenderSetupPlugin` and uses `HeadlessAssetsPlugin`. The VFX crate integrates:

```rust
// In game.rs:
if headless {
    app.add_plugins(RantzVfxPlugin::headless());
} else {
    app.add_plugins(RantzVfxPlugin::default());
}
```
