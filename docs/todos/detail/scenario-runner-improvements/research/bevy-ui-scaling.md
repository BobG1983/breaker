# Bevy 0.18.1 — UI Scaling API Research

**Verified against**: Bevy 0.18.1 (exact version used in this project)
**Sources**: docs.rs/bevy/0.18.1, raw GitHub source v0.18.1, official examples

---

## 1. Val Variants Available for Responsive Sizing

`bevy::ui::Val` (re-exported via `bevy::prelude::Val`) has these variants:

```rust
pub enum Val {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    VMin(f32),
    VMax(f32),
}
```

### How Each Variant Resolves (verified from `crates/bevy_ui/src/layout/convert.rs`)

The layout pass constructs a `LayoutContext { scale_factor: f32, physical_size: Vec2 }`.
`physical_size` is the physical window pixel size. `scale_factor` is
`camera.target_scaling_factor() * ui_scale.0`.

| Variant | Resolved value (Taffy units) | Notes |
|---|---|---|
| `Val::Px(v)` | `scale_factor * v` | Scaled by UiScale and DPI factor |
| `Val::Percent(v)` | `v / 100.0` of parent node dimension | Not scaled; relative to parent |
| `Val::Vw(v)` | `physical_size.x * v / 100.0` | Relative to window width — NOT divided by scale_factor |
| `Val::Vh(v)` | `physical_size.y * v / 100.0` | Relative to window height — NOT divided by scale_factor |
| `Val::VMin(v)` | `physical_size.min_element() * v / 100.0` | Relative to smaller dimension |
| `Val::VMax(v)` | `physical_size.max_element() * v / 100.0` | Relative to larger dimension |
| `Val::Auto` | Determined by Taffy/Flexbox context | No fixed pixel value |

**Critical gotcha**: `Val::Vw/Vh/VMin/VMax` use raw `physical_size` — they are NOT
divided by `scale_factor`. This means they are already in physical pixels, not logical
pixels. A 1000-pixel-wide window with UiScale 2.0 gives `Val::Vw(100.)` = 1000 physical
pixels, not 500. They respond to window resize automatically via `ComputedUiRenderTargetInfo`.

`Val::Percent` is relative to the **parent Node**, not the window (except for root nodes,
where the parent is the window itself).

---

## 2. UiScale Resource

```rust
// bevy::ui::UiScale (re-exported in bevy::prelude)
#[derive(Debug, Reflect, Resource, Deref, DerefMut)]
#[reflect(Resource, Debug, Default)]
pub struct UiScale(pub f32);

impl Default for UiScale {
    fn default() -> Self { Self(1.0) }
}
```

**Module path**: `bevy::ui::UiScale` / `bevy::prelude::UiScale`

**What it does**: A multiplier applied to the `scale_factor` used during layout.

The full scale factor passed to the layout system is:
```
layout_scale_factor = camera.target_scaling_factor() * ui_scale.0
```

This `scale_factor` is applied to `Val::Px` values during layout (see above).
It is also passed to the text pipeline, where a comment in `crates/bevy_ui/src/widget/text.rs`
confirms: `"scale_factor is already multiplied by UiScale"`.

**What UiScale affects**:
- `Val::Px` sizing and spacing — multiplied by `scale_factor`
- `TextFont::font_size` — YES, also scaled (confirmed by source comment)
- `Val::Vw/Vh/VMin/VMax` — NOT affected (use raw physical_size)
- `Val::Percent` — NOT affected (ratio-based)

**Usage pattern from official example** (`examples/ui/ui_scaling.rs`):

```rust
fn apply_scaling(
    time: Res<Time>,
    mut target_scale: ResMut<TargetScale>,
    mut ui_scale: ResMut<UiScale>,
) {
    ui_scale.0 = target_scale.current_scale();
}
```

Setting `ui_scale.0` uniformly scales all `Val::Px` and font sizes across every
UI node. It is a global multiplier — it cannot be applied per-node.

---

## 3. Font Scaling

`TextFont::font_size` is a plain `f32` in physical pixels (no viewport-relative
variant exists in Bevy 0.18.1):

```rust
// bevy::text::TextFont (re-exported in bevy::prelude)
pub struct TextFont {
    pub font: Handle<Font>,
    pub font_size: f32,   // vertical height of glyphs in pixels
    pub weight: FontWeight,
    pub font_smoothing: FontSmoothing,
    pub font_features: FontFeatures,
}
```

**No built-in mechanism** for viewport-relative font sizes in Bevy 0.18.1.

### Option A — UiScale (simplest, global)

Set `UiScale` to a value derived from the window size. Because `font_size` is
multiplied by `scale_factor` (which includes `UiScale`), all fonts scale uniformly:

```rust
fn update_ui_scale(
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        // Design resolution: 1920x1080
        let scale = (window.height() / 1080.0).min(window.width() / 1920.0);
        ui_scale.0 = scale;
    }
}
```

Run this in `Update`. All `Val::Px` and `font_size` values written for 1920×1080
will scale to the actual window. No individual node changes needed.

**Trade-off**: This changes how all `Val::Px` values behave. Any value intended to
be a fixed pixel size (e.g., a 1px border) also scales. And since `Val::Vw/Vh` are
NOT affected by UiScale, mixing Vw/Vh with Px in the same layout may give unexpected
results if UiScale != 1.0.

### Option B — Viewport-relative units (no resource, per-value)

Replace `Val::Px` with `Val::Vw` or `Val::Vh` for spacing and sizing. For fonts,
compute `font_size` at spawn time from the window dimensions:

```rust
fn spawn_menu(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    let scale = window.height() / 1080.0;
    let title_font_size = 173.0 * scale;

    commands.spawn((
        Node {
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            row_gap: Val::Vh(2.0),  // 2% of viewport height
            ..default()
        },
        // ...
    )).with_children(|parent| {
        parent.spawn((
            Text::new("BREAKER"),
            TextFont { font_size: title_font_size, ..default() },
        ));
    });
}
```

**Trade-off**: Does not respond to mid-session window resizes (UI is spawned once,
not re-spawned). Requires re-spawning or a reactive system to handle resize.

### Option C — Reactive font-size system (responds to resize)

A system that listens for window resize events and updates `TextFont::font_size`
on all text entities:

```rust
fn update_font_sizes_on_resize(
    mut resize_reader: EventReader<WindowResized>,
    mut text_query: Query<(&mut TextFont, &BaseFontSize)>,
) {
    for event in resize_reader.read() {
        let scale = (event.height / 1080.0).min(event.width / 1920.0);
        for (mut font, base) in &mut text_query {
            font.font_size = base.0 * scale;
        }
    }
}
```

This requires a `BaseFontSize(f32)` marker component to store the design-time size.
It handles mid-session resizes correctly.

---

## 4. Recommended Pattern for Bevy 0.18.1 Window-Resizable UI

Based on the official example and Bevy source, the clearest pattern is **UiScale
driven by window dimensions** (Option A). It requires the fewest changes and keeps
spawn code identical to the design-resolution values.

### Full pattern

```rust
// Resource: stores the design resolution
#[derive(Resource)]
struct DesignResolution {
    width: f32,
    height: f32,
}

// System: runs every frame (or on WindowResized events)
fn sync_ui_scale(
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    design: Res<DesignResolution>,
) {
    if let Ok(window) = windows.get_single() {
        let scale_x = window.width() / design.width;
        let scale_y = window.height() / design.height;
        // Use the smaller factor so UI always fits (letterboxed)
        ui_scale.0 = scale_x.min(scale_y);
    }
}

// Plugin wiring
app
    .insert_resource(DesignResolution { width: 1920.0, height: 1080.0 })
    .add_systems(Update, sync_ui_scale);
```

With this in place, all `Val::Px` values and all `font_size` values written for
1920×1080 will automatically scale to any window size.

### Constraints and gotchas for this project

1. **Val::Vw/Vh are NOT affected by UiScale**. Do not mix them with `Val::Px` in
   the same layout if UiScale != 1.0 — the two unit systems will be on different
   scales. Choose one approach per layout.

2. **Val::Percent is not affected by UiScale**. Root container
   `width: Val::Percent(100.0)` / `height: Val::Percent(100.0)` continues to work
   correctly regardless of UiScale.

3. **Border widths scale too**. A `border: Val::Px(1.0)` becomes `border: 0.5px`
   at UiScale 0.5. Consider whether thin borders should use `Val::Px` with a
   minimum (Bevy will not render sub-pixel borders — they become 0).

4. **UiScale default is 1.0**. If the sync system hasn't run yet on frame 0, the
   UI renders at the design resolution before correcting on the next frame. Add
   `Startup` as a second schedule or use `PreUpdate` ordering to minimize this.

5. **No per-node UiScale**. The resource is global. All UI scales together.

6. **bevy_egui is independent**. `bevy_egui` has its own DPI scale mechanism
   (`EguiSettings::scale_factor`). If the project uses egui for debug panels, it
   must be adjusted separately.

---

## 5. Summary Table

| Mechanism | What it scales | Responds to resize? | Per-node? |
|---|---|---|---|
| `UiScale` resource | `Val::Px` + `font_size` | Only if updated by system | No (global) |
| `Val::Vw/Vh/VMin/VMax` | Size only (not font) | Yes, automatically | Yes (per-value) |
| `Val::Percent` | Size only (not font) | Yes (relative to parent) | Yes (per-value) |
| Manual font-size system | `font_size` only | Yes (with resize event) | Yes |

**For this project**: UiScale driven by `window.height() / 1080.0` is the
lowest-friction fix. All existing spawn code can stay identical — it was written
assuming 1920×1080. The sync system normalizes UiScale so those pixel values
resolve correctly at any window size.

---

## Source References

- `Val` enum variants: `docs.rs/bevy/0.18.1/bevy/ui/enum.Val.html`
- `UiScale` struct: `docs.rs/bevy/0.18.1/bevy/ui/struct.UiScale.html`
- Layout conversion (how Val resolves): `crates/bevy_ui/src/layout/convert.rs` @ v0.18.1
- UiScale applied to scale_factor: `crates/bevy_ui/src/update.rs` (`propagate_ui_target_cameras`)
- Font size scaled by UiScale: `crates/bevy_ui/src/widget/text.rs` (comment: `"scale_factor is already multiplied by UiScale"`)
- Official UI scaling example: `examples/ui/ui_scaling.rs` @ v0.18.1
