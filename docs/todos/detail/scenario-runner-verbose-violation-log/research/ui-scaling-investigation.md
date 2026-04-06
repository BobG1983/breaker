## Behavior Trace: UI Scaling on Window Resize

### Summary

Every UI screen in the game uses a mix of `Val::Px(...)` (absolute pixels) for
spacing, sizing, and font sizes alongside `Val::Percent(...)` for root container
dimensions. The play area scales correctly on window resize because the camera
uses `ScalingMode::AutoMin`, which maps world-space coordinates to a virtual
viewport of 1920×1080 regardless of the physical window size. The UI layer has
no equivalent mechanism — Bevy's UI layout resolves `Val::Px` values against
physical window pixels directly, so any `Val::Px` value is fixed in physical-
pixel terms and does not scale.

---

### Camera Setup (the root cause)

**File:** `breaker-game/src/game.rs` — `spawn_camera`

```rust
Camera2d,
Projection::from(OrthographicProjection {
    scaling_mode: ScalingMode::AutoMin {
        min_width:  1920.0,
        min_height: 1080.0,
    },
    ..OrthographicProjection::default_2d()
}),
```

`ScalingMode::AutoMin` ensures the 2D world coordinates (play area, walls,
bolt, breaker, cells) always fill at least a 1920×1080 virtual viewport and
scale proportionally with the window. This is correct for gameplay entities.

Bevy's UI (`Node` / `Val`) is resolved independently in a separate layout pass
that operates in physical window pixels. `ScalingMode` on the camera has **no
effect** on UI layout. `Val::Percent` is relative to the parent node (or window
for root nodes), so it scales; `Val::Px` is always literal pixels, and does not
scale.

---

### PlayfieldConfig (play area — scales fine)

**File:** `breaker-game/src/shared/playfield.rs`
**RON:** `assets/config/defaults.playfield.ron`

```
width: 1440.0, height: 1080.0
```

These are **world-space units**, not pixels. They are used by physics and
collision systems to position entities. The camera's `ScalingMode::AutoMin`
ensures these world units map to the correct proportion of the screen regardless
of window size. The play area scales correctly.

---

### UI Root Nodes — All Screens

Every screen spawns a root `Node` with `width: Val::Percent(100.0)` and
`height: Val::Percent(100.0)`, which correctly fills the window at any size.
The problem is the absolute-pixel values used for sizing, spacing, and fonts
inside those root nodes.

---

### Catalog of Absolute-Pixel Values by Screen

#### 1. Loading Screen
**File:** `breaker-game/src/state/app/loading/systems/spawn_loading_screen.rs`

| Location | Value | Type |
|---|---|---|
| `row_gap` | `Val::Px(29.0)` | spacing |
| Loading bar background `width` | `Val::Px(720.0)` | hard-coded bar width |
| Loading bar background `height` | `Val::Px(43.0)` | hard-coded bar height |
| "Loading..." `font_size` | `43.0` | absolute pixels |

The loading bar is the most extreme case: its width and height are fixed at 720×43
physical pixels regardless of window size. On a 2560×1440 window it will look
small; on a 1280×720 window it may be acceptably sized but won't fill the
expected proportion of the screen.

#### 2. Main Menu
**File:** `breaker-game/src/state/menu/main/systems/spawn_main_menu.rs`
**Config:** `assets/config/defaults.mainmenu.ron`

All values flow through `MainMenuConfig`:

| Field | RON value | Usage |
|---|---|---|
| `title_font_size` | `173.0` | title "BREAKER" text |
| `menu_font_size` | `65.0` | "Play / Settings / Quit" items |
| `title_bottom_margin` | `86.0` | `Val::Px(config.title_bottom_margin)` |
| `menu_item_gap` | `22.0` | `Val::Px(config.menu_item_gap)` |
| Menu button `padding` | `Val::Px(43.0)` / `Val::Px(14.0)` | hardcoded in spawn code |

All font sizes and spacing are `f32` values passed directly to `font_size` and
`Val::Px`. There is no viewport-relative sizing path.

#### 3. Run Setup (Breaker Select)
**File:** `breaker-game/src/state/menu/start_game/systems/spawn_run_setup.rs`

| Location | Value | Type |
|---|---|---|
| Root `row_gap` | `Val::Px(40.0)` | spacing |
| Cards container `row_gap` | `Val::Px(16.0)` | spacing |
| Card button `padding` | `Val::Px(40.0)` / `Val::Px(16.0)` | spacing |
| Card `row_gap` | `Val::Px(8.0)` | spacing |
| "SELECT BREAKER" `font_size` | `72.0` | title |
| Breaker name `font_size` | `48.0` | per-card name |
| Description `font_size` | `24.0` | per-card description |
| Seed display `font_size` | `32.0` | seed text |
| Prompt `font_size` | `28.0` | bottom prompt |

#### 4. Pause Menu
**File:** `breaker-game/src/state/pause/systems/spawn_pause_menu.rs`

| Location | Value | Type |
|---|---|---|
| Root `row_gap` | `Val::Px(32.0)` | spacing |
| Title `margin` bottom | `Val::Px(24.0)` | spacing |
| "PAUSED" `font_size` | `72.0` | title |
| Menu items `font_size` | `36.0` | per-item text |

#### 5. Chip Select Screen
**File:** `breaker-game/src/state/run/chip_select/systems/spawn_chip_select.rs`
**Config:** `assets/config/defaults.chipselect.ron`

| Location | Value | Source |
|---|---|---|
| Root `row_gap` | `Val::Px(32.0)` | hardcoded |
| Card row `column_gap` | `Val::Px(24.0)` | hardcoded |
| Chip card `width` | `Val::Px(200.0)` | hardcoded |
| Chip card `height` | `Val::Px(280.0)` | hardcoded |
| Chip card `padding` | `Val::Px(16.0)` all sides | hardcoded |
| Chip card `row_gap` | `Val::Px(12.0)` | hardcoded |
| Chip card `border` | `Val::Px(2.0)` all sides | hardcoded |
| "CHOOSE A CHIP" `font_size` | `48.0` | hardcoded |
| Timer `font_size` | `config.timer_font_size` → `48.0` | RON |
| Card title `font_size` | `config.card_title_font_size` → `36.0` | RON |
| Card description `font_size` | `config.card_description_font_size` → `20.0` | RON |
| Prompt `font_size` | `24.0` | hardcoded |

The chip card dimensions (200×280 px) are the most visually significant: cards
are fixed-size boxes. At very large windows they will appear small and
undersized relative to the screen.

#### 6. HUD — Side Panels (in-node)
**File:** `breaker-game/src/state/run/node/hud/systems/spawn_side_panels.rs`

| Location | Value | Type |
|---|---|---|
| Left/right panel `padding` | `Val::Px(24.0)` all sides | spacing |
| Left panel `border` right | `Val::Px(1.0)` | border |
| Right panel `row_gap` | `Val::Px(12.0)` | spacing |
| Right panel `border` left | `Val::Px(1.0)` | border |
| "AUGMENTS" / "STATUS" `font_size` | `28.0` | section header |
| Divider "—" `font_size` | `18.0` | decorative |

Panel widths use `Val::Percent(12.5)` — they correctly scale with window width.
But the internal padding, borders, and font sizes are fixed.

#### 7. HUD — Timer Display (in-node)
**File:** `breaker-game/src/state/run/node/hud/systems/spawn_timer_hud.rs`
**Config:** `assets/config/defaults.timerui.ron`

| Location | Value | Source |
|---|---|---|
| Timer wrapper `padding` | `Val::Px(10.0)` / `Val::Px(4.0)` | hardcoded |
| Timer wrapper `border_radius` | `Val::Px(6.0)` | hardcoded |
| Timer `font_size` | `config.font_size` → `24.0` | RON |

#### 8. Run End Screen
**File:** `breaker-game/src/state/run/run_end/systems/spawn_run_end_screen/system.rs`

| Location | Value | Type |
|---|---|---|
| Root `row_gap` | `Val::Px(43.0)` | spacing |
| Title `font_size` | `130.0` | absolute |
| Subtitle `font_size` | `50.0` | absolute |
| Stats entries `font_size` | `28.0` | absolute |
| Flux earned `font_size` | `28.0` | absolute |
| Highlight entries `font_size` | `24.0` | absolute |
| Chip name entries `font_size` | `24.0` | absolute |
| Seed `font_size` | `20.0` | absolute |
| "Press Enter" prompt `font_size` | `43.0` | absolute |

---

### Highlight Text Popups (world-space, not UI)

**File:** `breaker-game/src/state/run/node/lifecycle/systems/spawn_highlight_text/system.rs`

Highlight popups ("CLUTCH CLEAR!", "MASS DESTRUCTION!", etc.) use `Text2d` and
`Transform`, **not** `Node`. They exist in 2D world space and are positioned via
`config.popup_base_y` / `config.popup_vertical_spacing` (world units). They
scale correctly because they live in world space and the camera's `ScalingMode`
applies to them. These are **not** part of the UI scaling problem.

---

### Root Cause Summary

```
Window resize
    ↓
Camera ScalingMode::AutoMin { min_width: 1920.0, min_height: 1080.0 }
    ↓
World-space entities (bolt, breaker, cells, walls, Text2d popups) — SCALE CORRECTLY
    ↓ (no connection)
Bevy UI layout pass — resolves Val::Px in physical window pixels — DOES NOT SCALE
```

The camera `ScalingMode` is a rendering concern. It does not feed into Bevy's
UI layout system. Every `Val::Px` value in every `Node` hierarchy is resolved
against the physical window pixel dimensions.

---

### What Scales vs. What Does Not

| Construct | Scales with window? | Reason |
|---|---|---|
| `Val::Percent(...)` on root `Node` width/height | Yes | Relative to parent/window |
| `Val::Px(...)` padding, gap, size | No | Physical pixels |
| `font_size: f32` on `TextFont` | No | Physical pixels |
| `Val::Px(...)` card `width`/`height` | No | Fixed pixel boxes |
| `Val::Px(...)` loading bar `width`/`height` | No | Fixed pixel boxes |
| `Text2d` + `Transform` in world space | Yes | `ScalingMode` applies |
| Gameplay entities (bolt, breaker) | Yes | `ScalingMode` applies |

---

### Scope of the Fix

To make UI scale with window size, every `Val::Px` and absolute `font_size` in
the UI layer needs a viewport-relative replacement. The two approaches available
in Bevy 0.18 are:

1. **`Val::Vw(...)` / `Val::Vh(...)`** — percentage of the viewport width/height.
   Replaces `Val::Px` for spacing, padding, and fixed-dimension elements.
2. **Computed font size** — since `TextFont::font_size` is a plain `f32` with no
   viewport-relative variant in Bevy 0.18, font sizes would need to be computed
   at runtime from window dimensions (e.g., read `Window` resource in the spawn
   systems and multiply a base size by `window.height / 1080.0`).

Alternatively, a single scale factor resource derived from the window size could
be inserted and read by all spawn systems to convert design-time pixel values to
window-relative values.

---

### Files to Change (full list)

| File | What needs changing |
|---|---|
| `breaker-game/src/state/app/loading/systems/spawn_loading_screen.rs` | Loading bar `Val::Px` width/height, `row_gap`, `font_size` |
| `breaker-game/src/state/menu/main/systems/spawn_main_menu.rs` | Padding `Val::Px(43.0)`/`Val::Px(14.0)` |
| `breaker-game/src/state/menu/main/resources.rs` | `title_bottom_margin`, `menu_item_gap`, `title_font_size`, `menu_font_size` fields used as `Val::Px`/`font_size` |
| `assets/config/defaults.mainmenu.ron` | Values will change if design-time → viewport-relative approach is adopted |
| `breaker-game/src/state/menu/start_game/systems/spawn_run_setup.rs` | All `Val::Px` spacing, all `font_size` literals |
| `breaker-game/src/state/pause/systems/spawn_pause_menu.rs` | All `Val::Px` spacing, all `font_size` literals |
| `breaker-game/src/state/run/chip_select/systems/spawn_chip_select.rs` | Card `Val::Px(200.0)`/`Val::Px(280.0)`, all `Val::Px` gaps/padding/border, hardcoded `font_size: 48.0`/`24.0` |
| `breaker-game/src/state/run/chip_select/resources.rs` | `card_title_font_size`, `card_description_font_size`, `timer_font_size` used directly as `font_size` |
| `assets/config/defaults.chipselect.ron` | Font size values |
| `breaker-game/src/state/run/node/hud/systems/spawn_side_panels.rs` | Panel `Val::Px` padding/border, `font_size` literals |
| `breaker-game/src/state/run/node/hud/systems/spawn_timer_hud.rs` | Wrapper `Val::Px` padding/border_radius |
| `breaker-game/src/state/run/node/hud/resources.rs` | `font_size` field used as `TextFont::font_size` |
| `assets/config/defaults.timerui.ron` | `font_size` value |
| `breaker-game/src/state/run/run_end/systems/spawn_run_end_screen/system.rs` | All `font_size` literals, `row_gap Val::Px(43.0)` |

### Files That Do NOT Need Changing (already scale-safe)

| File | Reason |
|---|---|
| `breaker-game/src/game.rs` — `spawn_camera` | Camera `ScalingMode` is correct for world-space |
| `breaker-game/src/shared/playfield.rs` | World-space units, not UI |
| `spawn_highlight_text/system.rs` | `Text2d` in world space, scales with camera |
| All gameplay entity spawn systems | World-space, camera handles scaling |
