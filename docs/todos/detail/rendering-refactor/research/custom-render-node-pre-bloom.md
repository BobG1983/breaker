# Custom Render Node Pre-Bloom (Bevy 0.18)

Verified against: Bevy 0.18.0 / 0.18.1 source and docs.rs.
Research date: 2026-03-30.

---

## 1. Exact Bevy 0.18 Core2d Render Graph Node Ordering

### Base graph (Core2dPlugin — `bevy_core_pipeline::core_2d`)

```
Node2d::StartMainPass
  → Node2d::MainOpaquePass
  → Node2d::MainTransparentPass
  → Node2d::EndMainPass
  → Node2d::StartMainPassPostProcessing   ← NEW in 0.17, anchor for post-process ordering
  → Node2d::Tonemapping
  → Node2d::EndMainPassPostProcessing
  → Node2d::Upscaling
```

`Wireframe` is in the enum but NOT in the core edge sequence.
`Bloom`, `PostProcessing`, `Fxaa`, `Smaa`, `ContrastAdaptiveSharpening` are in Node2d enum but added by their own plugins.

### Bloom (BloomPlugin — `bevy_post_process::bloom`)

In **Bevy 0.18**, Bloom moved from `bevy_core_pipeline` to `bevy_post_process`. Its render graph placement:

```rust
// Core2d
.add_render_graph_node::<ViewNodeRunner<BloomNode>>(Core2d, Node2d::Bloom)
.add_render_graph_edges(Core2d, (
    Node2d::StartMainPassPostProcessing,
    Node2d::Bloom,
    Node2d::Tonemapping,
))
```

**Critical: In 0.18, Bloom sits between `StartMainPassPostProcessing` and `Tonemapping`.**
(In 0.15 it was `EndMainPass → Bloom → Tonemapping` — this changed in 0.17.)

### PostProcessing node (ChromaticAberration, etc.)

```
Node2d::Bloom → Node2d::PostProcessing → Node2d::Tonemapping
```

### FullscreenMaterial (bevy_core_pipeline::fullscreen_material)

Runs at any position you configure via `node_edges()`. If not specifying sub_graph, it auto-detects 2D vs 3D. Default example places it after Tonemapping and before EndMainPassPostProcessing.

---

## 2. Full Node2d Enum Variants (Bevy 0.18)

```rust
// bevy::core_pipeline::core_2d::graph::Node2d
pub enum Node2d {
    MsaaWriteback,
    StartMainPass,
    MainOpaquePass,
    MainTransparentPass,
    EndMainPass,
    Wireframe,
    StartMainPassPostProcessing,  // anchor point for post-process ordering
    Bloom,
    PostProcessing,
    Tonemapping,
    Fxaa,
    Smaa,
    Upscaling,
    ContrastAdaptiveSharpening,
    EndMainPassPostProcessing,
}
```

`Core2d` is a unit struct with `#[derive(RenderSubGraph)]` at path `bevy::core_pipeline::core_2d::graph::Core2d`.

---

## 3. Pre-Bloom Custom Node — Correct Insertion Point

A ScreenFlash effect that must be bloomed needs to write to the HDR scene texture **before** `Node2d::Bloom` reads it.

**Target order:**
```
EndMainPass → StartMainPassPostProcessing → [ScreenFlashNode] → Bloom → Tonemapping
```

The custom node should be inserted between `StartMainPassPostProcessing` and `Bloom`:

```rust
render_app
    .add_render_graph_node::<ViewNodeRunner<ScreenFlashNode>>(
        Core2d,
        ScreenFlashLabel,
    )
    .add_render_graph_edges(
        Core2d,
        (
            Node2d::StartMainPassPostProcessing,
            ScreenFlashLabel,
            Node2d::Bloom,
        ),
    );
```

This ensures: scene is rendered → flash writes onto HDR texture → Bloom samples the combined result.

---

## 4. ViewNode Trait (Bevy 0.18)

### Signature

```rust
// bevy::render::render_graph::ViewNode
pub trait ViewNode {
    type ViewQuery: ReadOnlyQueryData;

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext<'w>,
        view_query: <Self::ViewQuery as WorldQuery>::Item<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError>;

    fn update(&mut self, _world: &mut World) {} // optional
}
```

### ViewNodeRunner

Wrap the struct in `ViewNodeRunner<T>` when adding to the render graph. `ViewNodeRunner` automatically runs the node once per view entity matching `ViewQuery`.

```rust
render_app.add_render_graph_node::<ViewNodeRunner<ScreenFlashNode>>(Core2d, ScreenFlashLabel)
```

### Label

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ScreenFlashLabel;
```

---

## 5. How a ViewNode Accesses and Writes the HDR Scene Texture

### The ViewTarget double-buffer pattern

`ViewTarget` manages two HDR textures that flip between "current" and "other" for post-processing. Bloom reads without consuming (`main_texture_view()` directly), but a custom node that **modifies** the texture must use `post_process_write()`.

### `post_process_write()` — for read-modify-write

```rust
pub fn post_process_write(&self) -> PostProcessWrite<'_>

pub struct PostProcessWrite<'a> {
    pub source: &'a TextureView,  // current main texture (read from here)
    pub destination: &'a TextureView,  // next main texture (write here)
}
```

Calling `post_process_write()` internally flips the ViewTarget so `destination` becomes the new "current." You must copy source → destination (with your modification), otherwise the texture is lost.

### `ViewTarget::TEXTURE_FORMAT_HDR`

```rust
pub const TEXTURE_FORMAT_HDR: TextureFormat = TextureFormat::Rgba16Float;
```

**CRITICAL**: When Bloom is enabled, the camera uses HDR mode. A custom render pipeline for the flash must use `ViewTarget::TEXTURE_FORMAT_HDR` as the color target format, NOT `TextureFormat::bevy_default()`. Mixing formats causes a wgpu validation crash (confirmed in issue #21516).

### Accessing ViewTarget in ViewQuery

```rust
impl ViewNode for ScreenFlashNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ScreenFlashSettings,  // your flash data component
    );

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext<'w>,
        (view_target, flash_settings): <Self::ViewQuery as WorldQuery>::Item<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let post_process = view_target.post_process_write();
        // post_process.source = current HDR scene texture
        // post_process.destination = write your result here
        // ...
        Ok(())
    }
}
```

---

## 6. Additive Blending in a Custom Render Pass

To additively blend the flash color onto the scene, set up the render pipeline with additive blend state. This is done via the pipeline descriptor, NOT via `AlphaMode2d` (which has no additive variant in 0.18).

### In the pipeline descriptor

```rust
use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState};

let blend_state = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
};
```

However: using `post_process_write()` gives source+destination textures. The node renders a fullscreen triangle that samples `source` and outputs `source_color + flash_color` to `destination`. This is equivalent to additive blending but implemented in the fragment shader rather than the GPU blend unit.

**Fragment shader approach (recommended for ViewNode):**
```wgsl
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(source_texture, source_sampler, in.uv);
    let flash = uniforms.flash_color; // HDR color e.g. vec4(2.0, 2.0, 2.0, 1.0)
    // Additive: output = scene + flash * flash_alpha
    return scene_color + flash * flash.a;
}
```

---

## 7. AlphaMode2d — No Additive Variant

`AlphaMode2d` (for `ColorMaterial` / `Mesh2d`) has only three variants:
- `Opaque`
- `Mask(f32)`
- `Blend`

**There is no `Add`/`Additive` variant in Bevy 0.18.** This is confirmed — 2D does not support additive blending through the sprite/mesh material system.

---

## 8. FullscreenMaterial Approach

### What it is

`FullscreenMaterial` (new in 0.18, `bevy::core_pipeline::fullscreen_material`) is a high-level abstraction that wraps the ViewNode pattern. Implements the trait instead of manually writing render graph code.

### Trait

```rust
pub trait FullscreenMaterial:
    Component + ExtractComponent + Clone + Copy + ShaderType + WriteInto + Default
{
    fn fragment_shader() -> ShaderRef;                      // required
    fn node_edges() -> Vec<InternedRenderLabel>;            // required
    fn sub_graph() -> Option<InternedRenderSubGraph> { None } // optional
    fn node_label() -> impl RenderLabel { ... }              // has default
}
```

### node_edges() controls position

```rust
fn node_edges() -> Vec<InternedRenderLabel> {
    vec![
        Node2d::StartMainPassPostProcessing.intern(),
        Self::node_label().intern(),
        Node2d::Bloom.intern(),
    ]
}
```

This would place the FullscreenMaterial before Bloom — making it a pre-bloom effect.

### Limitation for additive blending

`FullscreenMaterial` uses a fullscreen triangle that reads `source` and writes `destination` (via post_process_write semantics). The fragment shader must implement the additive logic manually. The pipeline blend state is fixed by the internal implementation (likely REPLACE/ALPHA_BLENDING). Check source to confirm if a custom pipeline state can be provided.

---

## 9. Alternative: Fullscreen Entity in Scene

### The idea

Spawn a `Mesh2d` fullscreen quad with a `ColorMaterial` in the Transparent2d phase, before EndMainPass. Bloom reads the scene after EndMainPass, so this entity's color would be bloomed naturally.

### Why it mostly works

- Entity renders in `MainTransparentPass` (before bloom)
- HDR color values > 1.0 get bloomed
- No render graph customization needed

### Why it fails for additive blending

- `AlphaMode2d` has no `Add` variant — only `Blend` (standard alpha over) and `Opaque`
- Cannot achieve true additive flash (src=One, dst=One) through the sprite/mesh system

### Why camera-relative sizing is awkward

- A fullscreen quad must be sized relative to the camera's orthographic projection size
- Must track camera zoom/viewport changes
- Z-ordering conflicts: must be behind game entities or use render layers

### Verdict

The in-scene entity approach works for an **opaque color overlay** (`AlphaMode2d::Opaque` with alpha channel in shader) or a standard alpha-blended overlay, but cannot achieve true additive blending in Bevy 0.18. For a screen flash that needs to additively brighten the scene (preserve all scene colors + add flash), you need the ViewNode approach.

---

## 10. Implementation Complexity Comparison

| Approach | Complexity | Additive blend | Pre-bloom | Notes |
|---|---|---|---|---|
| ViewNode (custom) | High | Yes (shader) | Yes | Full control, ~200 lines Rust + WGSL |
| FullscreenMaterial | Medium | Yes (shader) | Yes (via node_edges) | Less boilerplate, new in 0.18 |
| In-scene entity | Low | No (AlphaMode2d lacks Add) | Yes | Easiest but no true additive |

**Recommended approach**: `FullscreenMaterial` if the default pipeline state is acceptable. `ViewNode` if you need custom pipeline configuration or the FullscreenMaterial pipeline state is not overridable.

---

## 11. Key Use Statements for Implementation

```rust
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_resource::{
    BlendComponent, BlendFactor, BlendOperation, BlendState,
    BindGroupLayout, PipelineCache, RenderPipelineDescriptor, TextureFormat,
};
use bevy::render::view::{ViewTarget, PostProcessWrite};
use bevy::render::renderer::RenderContext;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::post_process::bloom::Bloom;  // Bloom moved to bevy_post_process in 0.17
```

---

## 12. Critical Gotchas

1. **HDR format**: Use `ViewTarget::TEXTURE_FORMAT_HDR` (`Rgba16Float`) in your pipeline when Bloom is active, not `TextureFormat::bevy_default()`. Mixing causes wgpu validation errors (issue #21516).

2. **Bloom position changed in 0.17**: In 0.15, Bloom was `EndMainPass → Bloom → Tonemapping`. In 0.18, it is `StartMainPassPostProcessing → Bloom → Tonemapping`. All pre-bloom custom nodes must target `StartMainPassPostProcessing` as predecessor.

3. **post_process_write semantics**: Calling `post_process_write()` flips the double buffer. The caller MUST write `source` to `destination` (with modifications). If you only write a solid color to `destination` without sampling `source`, the scene is wiped.

4. **Bloom reads main_texture_view() directly**: Bloom does NOT use post_process_write(). It reads `view_target.main_texture_view()` as input and writes to its own intermediate textures. This means a custom node that calls `post_process_write()` before Bloom correctly feeds the modified HDR texture into Bloom.

5. **AlphaMode2d lacks Add**: Cannot achieve additive blending on sprites/meshes in Bevy 0.18. Must use a custom render pass.

6. **FullscreenMaterial is 0.18-new**: Not available in 0.15 or 0.17 — verify it exists in this project's exact version (0.18.x).

7. **Bloom moved to bevy_post_process in 0.17**: Import path is `bevy::post_process::bloom::Bloom`, not `bevy::core_pipeline::bloom::Bloom`.
