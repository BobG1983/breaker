# 5s: Screens

**Goal**: Overhaul all non-gameplay screens to match the visual identity.

Architecture: `docs/architecture/rendering/screens.md`

## Shared Principles

- Each screen is a module in `screen/`
- Entity lifecycle: spawn on `OnEnter`, despawn on `OnExit` via cleanup markers
- VFX via `rantzsoft_vfx` primitives (ExecuteRecipe, AttachVisuals, modifiers)
- Text rendering: Bevy `Text2d` with monospace font, GlitchText overlay where appropriate
- Input handling is per-screen — menus may use Bevy UI (Node, Button, Interaction) or world-space entities with cursor-to-world input, whichever fits

## What to Build

### 1. Main Menu — Interactive Idle

- Playfield visible with grid background, walls, no cells
- Breaker + bolt fully interactive (player can move and bump casually)
- Menu options as world-space text entities with glow effects
- Cool palette (early-run temperature), low particle density

### 2. Run-End Screen (Hybrid, DR-2)

**Victory**: Splash treatment — stats slam in with energy effects, screen shake per reveal
**Defeat**: Hologram treatment — floating holographic display, stats appear one by one, calm/contemplative

Both display: run outcome, nodes cleared, highlights, flux earned, run seed.

Context-sensitive defeat:
- Early death (nodes 1-3): minimal fanfare, quick summary
- Late death (nodes 6+): show what was forming, "almost unlocked" evolution teases
- Spectacular death (high highlight count): highlight reel of peak moments

### 3. Breaker Select (Run Setup)

- Per-archetype visual preview via `AttachVisuals` (shape, color, aura, trail visible)
- Selection highlights with glow modifiers
- Bolt type preview alongside breaker

### 4. Pause Menu

- `TriggerDesaturation` on camera to dim the game
- GlitchText treatment on "PAUSED"
- Menu options as world-space text
- Game entities visible but dimmed beneath

### 5. Loading Screen

- Minimal: styled progress indicator + game logo
- Void background with subtle grid, cool temperature

## Dependencies

- **Requires**: 5c (crate), 5d (post-processing: desaturation for pause), 5f (temperature), 5h (breaker visuals for select screen), 5o (GlitchText), 5q (HUD typography)
- DR-2 resolved: hybrid run-end

## Verification

- Main menu is interactive (bolt bounces, breaker moves)
- Run-end shows correct content with chosen style per outcome
- Defeat shows "almost unlocked" teases when applicable
- Breaker select shows archetype visual previews with aura/trail
- Pause overlays desaturation on gameplay
- All existing tests pass
