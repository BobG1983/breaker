---
name: FullscreenMaterial API — Bevy 0.18.1
description: Verified FullscreenMaterial trait, FullscreenMaterialPlugin, ViewTarget post-process ping-pong, Node2d graph ordering, blend state limitations
type: reference
---

# FullscreenMaterial API — Bevy 0.18.1

Verified against: `docs.rs/bevy_core_pipeline/0.18.0/src/bevy_core_pipeline/fullscreen_material.rs.html`

## Module Path

```rust
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::core_pipeline::core_2d::graph::Node2d;
```

## Trait Signature (complete)

```rust
pub trait FullscreenMaterial:
    Component + ExtractComponent + Clone + Copy + ShaderType + WriteInto + Default
{
    fn fragment_shader() -> ShaderRef;                          // required
    fn node_edges() -> Vec<InternedRenderLabel>;                // required
    fn sub_graph() -> Option<InternedRenderSubGraph> { None }  // default: auto-detect from camera
    fn node_label() -> impl RenderLabel {                       // default: FullscreenMaterialLabel(type_name::<Self>())
        FullscreenMaterialLabel(type_name::<Self>())
    }
}
```

`FullscreenMaterialLabel` is a PRIVATE struct in bevy_core_pipeline — you cannot reference it from outside the crate. Override `node_label()` with your own public `#[derive(RenderLabel)]` type if you need to reference it from other effects' `node_edges()`.

## Multiple Types Per Camera

YES — works correctly. Each `FullscreenMaterialPlugin<T>` is independent:
- Unique node label via `type_name::<Self>()`
- Unique `FullscreenMaterialPipeline<T>` resource
- Independent render node in the graph

`post_process_write()` ping-pongs the ViewTarget textures on each call — sequential nodes chain correctly.

## node_edges() Mechanics

Uses `.windows(2)` to call `try_add_node_edge(a, b)` for consecutive pairs. `EdgeAlreadyExists` is silently ignored (safe to have overlapping declarations across multiple effects).

To place effect between A and C: `vec![A.intern(), MyLabel.intern(), C.intern()]`

## Blend State — HARDCODED, NO OVERRIDE

`blend: None` — hardcoded in `init_pipeline`. There is NO `specialize()` method on `FullscreenMaterial`. Cannot be changed.

**For additive blending:** implement in the fragment shader by sampling the source texture and adding the effect color manually. `post_process_write()` provides source as sampler input.

## HDR Support

Two pipelines created at `RenderStartup`:
- LDR: `TextureFormat::bevy_default()` (typically `Bgra8UnormSrgb`)
- HDR: `TextureFormat::Rgba16Float` (= `ViewTarget::TEXTURE_FORMAT_HDR`)

Selected automatically via `view_target.is_hdr()` at render time.

## Core2d Graph (with "2d" feature only)

```
StartMainPass → MainOpaquePass → MainTransparentPass → EndMainPass
  → StartMainPassPostProcessing → Tonemapping → EndMainPassPostProcessing → Upscaling
```

`Node2d::Bloom` exists as enum variant BUT is NOT registered in the graph with "2d" feature only. Using it in `node_edges()` will panic. Use `Node2d::Tonemapping` as pre-tonemapping anchor.

Pre-tonemapping (HDR data): between `StartMainPassPostProcessing` and `Tonemapping`
Post-tonemapping (LDR data): between `Tonemapping` and `EndMainPassPostProcessing`

## sub_graph() None vs Some

- `None` (default): node added in `ExtractSchedule` on `Added<T>` — one-frame delay on first activation
- `Some(Core2d.intern())`: node registered at plugin build() startup — no delay, always in graph

Use `Some(Core2d.intern())` to avoid the one-frame delay when the component is added mid-session.

## Plugin Registration

```rust
app.add_plugins(FullscreenMaterialPlugin::<MyEffect>::default());
```

Add to camera entity:

```rust
commands.spawn((Camera2d, MyEffect::default()));
```
