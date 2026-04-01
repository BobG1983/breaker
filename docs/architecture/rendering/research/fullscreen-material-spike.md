# Spike: FullscreenMaterial API Viability — Bevy 0.18.1

**Verdict: PARTIAL — usable for most effects, BLOCKED on additive blending.**

---

## Bevy Version

`bevy = "0.18.1"`, `features = ["2d", "serialize"]`

All findings are sourced from:
- `docs.rs/bevy_core_pipeline/0.18.0/src/bevy_core_pipeline/fullscreen_material.rs.html`
- `docs.rs/bevy/0.18.0/bevy/render/view/struct.ViewTarget.html`
- `docs.rs/bevy/0.18.0/bevy/core_pipeline/core_2d/graph/enum.Node2d.html`
- `docs.rs/bevy_core_pipeline/0.18.0/bevy_core_pipeline/core_2d/graph/enum.Node2d.html`

---

## What FullscreenMaterial Actually Is

`bevy::core_pipeline::fullscreen_material` exports:

- `FullscreenMaterial` — a trait (not `Material`)
- `FullscreenMaterialPlugin<T: FullscreenMaterial>` — a generic plugin

**Module path:** `bevy::core_pipeline::fullscreen_material` (NOT `bevy::render::render_resource`)

**Trait bounds:**

```rust
pub trait FullscreenMaterial:
    Component + ExtractComponent + Clone + Copy + ShaderType + WriteInto + Default
{
    // Required
    fn fragment_shader() -> ShaderRef;
    fn node_edges() -> Vec<InternedRenderLabel>;

    // Provided (with defaults)
    fn sub_graph() -> Option<InternedRenderSubGraph> { None }
    fn node_label() -> impl RenderLabel {
        FullscreenMaterialLabel(type_name::<Self>())  // unique per type
    }
}
```

---

## Question 1: Multiple FullscreenMaterial Types on One Camera

**VERDICT: YES — works correctly.**

Each `FullscreenMaterialPlugin<T>` is parameterized by `T` via `PhantomData<T>`. Each type gets:

- Its own independent `ViewNodeRunner<FullscreenMaterialNode<T>>` node in the render graph
- Its own unique `node_label()` derived from `type_name::<Self>()` — guaranteed distinct per type
- Its own `FullscreenMaterialPipeline<T>` resource (two variants: LDR and HDR)

When `sub_graph()` returns `None` (the default), a system in `ExtractSchedule` fires on `Added<T>` and adds the node to `Core2d` or `Core3d` based on which camera component the entity has. Different types each run this independently.

`post_process_write()` is called inside each node's run(). This flips the `ViewTarget`'s internal texture pair (ping-pong): the current main texture becomes `source`, the alternate becomes `destination`. Each sequential node call advances the ping-pong. **This chains correctly** — the output of node A becomes the input of node B, as long as the render graph orders them correctly (see Question 2).

**Constraint:** Each `T` must be a distinct Rust type. Multiple instances of the same `T` on one camera would conflict on node label. Each effect must be its own struct.

---

## Question 2: Custom Pipeline Ordering via node_edges()

**VERDICT: YES — works, with caveats about Bloom availability.**

### How node_edges() works

The plugin uses `.windows(2)` over the returned `Vec<InternedRenderLabel>`:

```rust
for window in T::node_edges().windows(2) {
    let [a, b] = window else { break; };
    let Err(err) = graph.try_add_node_edge(*a, *b) else { continue; };
    match err {
        RenderGraphError::EdgeAlreadyExists(_) => {}  // silently ignored
        _ => panic!("{err:?}"),
    }
}
```

To place an effect between nodes A and C, return `vec![A.intern(), MyLabel, C.intern()]`. This adds edges A→MyLabel and MyLabel→C.

**Example — pre-bloom (on HDR data):**

```rust
fn node_edges() -> Vec<InternedRenderLabel> {
    vec![
        Node2d::StartMainPassPostProcessing.intern(),
        FlashMaterial::node_label().intern(),
        Node2d::Bloom.intern(),
    ]
}
```

### Caveat: Bloom in the "2d" feature set

`Node2d::Bloom` exists as an enum variant (it implements `RenderLabel`). **However, the bloom node may not be registered in the render graph** when using `features = ["2d"]` only. The `bevy_core_pipeline` crate has no `bloom` module and no `bloom` feature flag. The Core2d graph is built with: `StartMainPassPostProcessing → Tonemapping → EndMainPassPostProcessing → Upscaling`. Bloom is not wired in by default.

**Risk:** If Bloom is not registered when `features = ["2d"]`, using `Node2d::Bloom` in `node_edges()` will panic (via `try_add_node_edge` returning a non-`EdgeAlreadyExists` error). The `"3d"` feature enables `bevy_pbr` which registers Bloom; `"2d"` alone does not.

**What to use instead for pre-tonemapping ordering with "2d":**

```rust
fn node_edges() -> Vec<InternedRenderLabel> {
    vec![
        Node2d::StartMainPassPostProcessing.intern(),
        FlashMaterial::node_label().intern(),
        Node2d::Tonemapping.intern(),
    ]
}
```

This places the effect between `StartMainPassPostProcessing` and `Tonemapping`, which is always registered and always on HDR data (before tonemapping).

### The Core2d Graph (confirmed, "2d" feature)

```
StartMainPass
  → MainOpaquePass
  → MainTransparentPass
  → EndMainPass
  → StartMainPassPostProcessing
  → Tonemapping
  → EndMainPassPostProcessing
  → Upscaling
```

Post-tonemapping effects go between `Tonemapping` and `EndMainPassPostProcessing` (or later). Pre-tonemapping effects go between `StartMainPassPostProcessing` and `Tonemapping`.

---

## Question 3: Custom Blend State

**VERDICT: BLOCKED — additive blending is impossible through FullscreenMaterial.**

The `init_pipeline<T>` function constructs the `ColorTargetState` with `blend: None` hardcoded:

```rust
ColorTargetState {
    format: TextureFormat::bevy_default(),
    blend: None,                     // hardcoded — no override path
    write_mask: ColorWrites::ALL,
}
```

This is duplicated to the HDR variant (only the format changes). There is **no `specialize()` method** on `FullscreenMaterial`. There is no pipeline specialization hook of any kind exposed to the user. The pipeline is initialized once in `RenderStartup` via `init_pipeline::<T>` and is fixed.

**Consequence for screen flash:** Additive blending (`scene_color + flash_color`) is impossible through `FullscreenMaterial`. You cannot do `BlendState::PREMULTIPLIED_ALPHA_BLENDING` or a custom additive blend.

**Workaround options:**

1. **Implement additive blending in the shader itself.** The flash shader reads `scene_color` from the source texture and outputs `scene_color + flash_color * flash_intensity` directly. Since the write replaces the destination entirely (no GPU blend unit), the shader must perform the blend math. This works because `post_process_write()` gives you the source texture as a sampler input.

2. **Custom ViewNode.** Implement `ViewNode` manually for the flash effect only, constructing the pipeline with the exact `BlendState` needed. This is the full fallback path.

Option 1 (shader-side additive) is simpler and achieves the same visual result. It is the recommended path.

---

## Question 4: ViewTarget::TEXTURE_FORMAT_HDR

**VERDICT: YES — HDR is fully supported.**

```rust
pub const TEXTURE_FORMAT_HDR: TextureFormat = TextureFormat::Rgba16Float;
```

`FullscreenMaterialPlugin` creates two pipelines at startup:
- LDR: `TextureFormat::bevy_default()` (typically `Bgra8UnormSrgb`)
- HDR: `TextureFormat::Rgba16Float`

The `ViewNode` selects at runtime:

```rust
let pipeline_id = if view_target.is_hdr() {
    fullscreen_pipeline.pipeline_id_hdr
} else {
    fullscreen_pipeline.pipeline_id
};
```

Any effect placed before `Tonemapping` operates on HDR (`Rgba16Float`) data, which is correct for a flash that must affect bloom. The selection is automatic — no action needed from the implementor.

---

## Effect-by-Effect Assessment

| Effect | Placement | Blend needed | Verdict |
|---|---|---|---|
| Screen flash (pre-bloom) | Between `StartMainPassPostProcessing` and `Tonemapping` | Additive | BLOCKED — use shader-side blend |
| Radial distortion | Between `Tonemapping` and `EndMainPassPostProcessing` | Replace | WORKS |
| Chromatic aberration | Between `Tonemapping` and `EndMainPassPostProcessing` | Replace | WORKS |
| Desaturation | Between `Tonemapping` and `EndMainPassPostProcessing` | Replace | WORKS |
| Vignette | Between `Tonemapping` and `EndMainPassPostProcessing` | Replace (darken) | WORKS — darken in shader |
| CRT overlay | After `EndMainPassPostProcessing` or before `Upscaling` | Replace | WORKS |
| Collapse/rebuild transition | Conditional — between `Tonemapping` and `EndMainPassPostProcessing` | Replace | WORKS |

**Note on ordering between post-tonemapping effects:** When multiple effects slot between `Tonemapping` and `EndMainPassPostProcessing`, they must chain via `node_edges()`. Example for three effects A → B → C:

```rust
// Effect A's node_edges():
vec![Node2d::Tonemapping.intern(), A::node_label().intern(), B::node_label().intern()]

// Effect B's node_edges():
vec![A::node_label().intern(), B::node_label().intern(), C::node_label().intern()]

// Effect C's node_edges():
vec![B::node_label().intern(), C::node_label().intern(), Node2d::EndMainPassPostProcessing.intern()]
```

`EdgeAlreadyExists` is silently ignored, so overlapping edge declarations are safe.

---

## Summary of Risks and Required Pivots

### Risk 1 (LOW): Additive blend for screen flash
- **Impact:** Flash effect only
- **Resolution:** Implement blend math in the fragment shader (read source texture, output `source + flash * intensity`). No custom ViewNode needed. This is idiomatic.

### Risk 2 (MEDIUM): Bloom node availability with "2d" feature
- **Impact:** If Bloom is not registered, using `Node2d::Bloom` in `node_edges()` panics at startup
- **Resolution:** Place pre-tonemapping effects between `StartMainPassPostProcessing` and `Node2d::Tonemapping` instead of `Node2d::Bloom`. The effect still operates on HDR data before tonemapping, which is sufficient.
- **Action needed:** Do NOT use `Node2d::Bloom` in `node_edges()`. Use `Node2d::Tonemapping` as the downstream anchor for pre-tonemapping effects.

### Risk 3 (LOW): Effect ordering between multiple post-tonemapping effects
- **Impact:** If `node_edges()` chains are set up incorrectly, nodes may execute in wrong order
- **Resolution:** Each effect must explicitly include its neighbors in `node_edges()`. The `windows(2)` mechanism requires the complete chain — each effect owns only its immediate neighbors.

### Non-Risk: Custom ViewNode fallback
- Custom `ViewNode` is NOT required for any of the planned effects given the above resolutions. All 6-7 effects can use `FullscreenMaterialPlugin`.

---

## Correct Usage Pattern

```rust
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::core_pipeline::core_2d::graph::Node2d;
use bevy::render::render_graph::InternedRenderLabel;

// Define a label for this effect (required for cross-effect ordering)
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ScreenFlashLabel;

#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
struct ScreenFlash {
    color: Vec4,
    intensity: f32,
    _padding: Vec3,
}

impl FullscreenMaterial for ScreenFlash {
    fn fragment_shader() -> ShaderRef {
        "shaders/screen_flash.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        // Place BEFORE tonemapping (on HDR data). Do NOT use Node2d::Bloom.
        vec![
            Node2d::StartMainPassPostProcessing.intern(),
            ScreenFlashLabel.intern(),
            Node2d::Tonemapping.intern(),
        ]
    }

    fn node_label() -> impl RenderLabel {
        ScreenFlashLabel
    }
}
```

In the shader (`screen_flash.wgsl`), implement additive blend manually:

```wgsl
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(screen_texture, texture_sampler, in.uv);
    let flash = material.color * material.intensity;
    // Additive blend (scene + flash), preserving HDR values
    return vec4(scene_color.rgb + flash.rgb, scene_color.a);
}
```

Register in the app:

```rust
app.add_plugins(FullscreenMaterialPlugin::<ScreenFlash>::default());
```

Add to camera entity:

```rust
commands.spawn((Camera2d, ScreenFlash::default()));
```

---

## What Needs Further Investigation

1. **`node_label()` return type.** The default returns `impl RenderLabel` (opaque). If you need to reference your node label from another effect's `node_edges()`, you must override `node_label()` to return a concrete, publicly accessible label type. The default `FullscreenMaterialLabel(&'static str)` is a private struct and cannot be used outside the crate.

2. **`ShaderType + WriteInto` derive requirements.** These require `encase` derives. Verify the `encase` dependency is available via Bevy's re-export before assuming derive macros work. Fields must be 16-byte aligned.

3. **`sub_graph()` vs default `None` path.** The default (`None`) path dynamically adds the node in `ExtractSchedule` on `Added<T>`. This means the node is added the first frame the component appears, not at startup. There is a one-frame delay before the effect renders. If this causes visual artifacts on effect activation, use `sub_graph()` instead (returns `Core2d.intern()`), which registers at startup.
