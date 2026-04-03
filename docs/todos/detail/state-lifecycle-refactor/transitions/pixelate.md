# Pixelate Transition

Post-processing effect that reduces screen resolution by sampling into progressively larger blocks. Uses Bevy 0.18's `FullscreenMaterial` trait — no manual render graph wiring or camera render targets.

## PixelateOut
Block size eases from 1px (full res) to a maximum size where the entire screen is one solid block of the overlay color. The screen content is downsampled — each block shows the nearest-neighbor sample of the pixels it covers. At maximum block size, snap to the solid overlay color.

## PixelateIn
Reverse — block size eases from maximum to 1px. Starts from solid overlay color, progressively reveals full resolution.

## PixelateOutIn
PixelateOut → state change → PixelateIn. Duration splits `TransitionConfig.duration` across Out and In phases.

## Block Size Progression
- Minimum: 1px (no pixelation — identity pass)
- Maximum: viewport height (single block)
- Easing curve maps to block size — the easing handles feel naturally
- Block size snaps to integer values (no sub-pixel blocks)

## Sampling
- Nearest-neighbor (not averaging) — gives the chunky retro look
- Sample point: center of each block
- UV coordinates quantized to block grid in the fragment shader

## Implementation — FullscreenMaterial

Unlike overlay-based transitions (Fade, Dissolve, etc.), Pixelate is a **post-processing effect** that reads the rendered screen texture. Bevy 0.18 provides `FullscreenMaterial` for exactly this.

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
pub struct PixelateEffect {
    pub block_size: f32,
    pub color: Vec4,       // overlay color for the final solid state (as linear RGBA)
    pub _padding: Vec2,    // ShaderType 16-byte alignment
}

impl FullscreenMaterial for PixelateEffect {
    fn fragment_shader() -> ShaderRef {
        "shaders/pixelate.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::StartMainPassPostProcessing.intern(),
            PixelateNode.intern(),
            Node2d::Tonemapping.intern(),
        ]
    }

    fn sub_graph() -> Option<InternedRenderSubGraph> {
        Some(Core2d.intern())
    }

    fn node_label() -> impl RenderLabel {
        PixelateNode
    }
}
```

- **start**: Insert `PixelateEffect` component on the camera with `block_size = 1.0` (identity). Register `FullscreenMaterialPlugin::<PixelateEffect>` at app startup.
- **run**: Each frame, sample easing curve → compute block size → update `PixelateEffect.block_size` on the camera component. At max block size, shader snaps to solid overlay color.
- **end**: Set `block_size = 1.0` (identity pass — no visible effect). The component stays on the camera; the node is always in the render graph.
- Fragment shader (`pixelate.wgsl`): reads `screen_texture` at `@group(0) @binding(0)`, quantizes UV to block grid, samples nearest-neighbor. When `block_size >= viewport_height`, outputs solid `color`.
- All timing uses `Time<Real>` — virtual time is paused during transitions
- No `GlobalZIndex` needed — this is a render graph node, not an overlay entity
- No overlay entity to spawn/despawn — just a camera component update

## Gotchas
- `ShaderType` requires 16-byte minimum struct size — pad with `Vec2` or `Vec3`
- `blend: None` is hardcoded in `FullscreenMaterial` — fine for pixelate (fully opaque output)
- Node is always in the render graph when `sub_graph() = Some(...)`. `block_size = 1.0` is the "disabled" state.
- Safe node anchors: `StartMainPassPostProcessing` → your node → `Tonemapping`. Do NOT reference `Node2d::Bloom` — not registered without the bloom feature.
