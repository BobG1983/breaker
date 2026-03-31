# Screen Rendering Principles

Architecture principles shared across all screens. Per-screen entity layouts, shaders, and animation sequences will be designed when each screen is implemented (steps 5s substeps).

## Shared Principles

1. **Each screen is a module in `screen/`.** `screen/main_menu/`, `screen/run_end/`, `screen/pause/`, etc.
2. **Entity lifecycle:** Spawn entities on `OnEnter(GameState::*)`, despawn on `OnExit(GameState::*)` via cleanup markers. No persistent screen entities across state changes.
3. **VFX via rantzsoft_vfx primitives.** Screens use `ExecuteRecipe`, `AttachVisuals`, and modifier messages — same tools as gameplay domains. No screen-specific rendering infrastructure.
4. **Text rendering:** Bevy `Text2d` with monospace font for in-world text. Glitch effects via GlitchText overlay where appropriate (titles, highlight labels).
5. **Input handling is per-screen.** Menus may use Bevy UI (Node, Button, Interaction) or world-space entities with cursor-to-world input — whichever fits the screen. The "no UI nodes" rule applies to the diegetic HUD (`screen/playing/hud/`), not to menu screens. Menus that need standard input handling (hover, click, keyboard nav) can use Bevy UI styled to match the aesthetic.

## Per-Screen Design Direction

### Main Menu

Interactive idle scene. Game entities (breaker, bolts, cells) playing autonomously in the background. Menu options as world-space text entities with glow effects. Selection via input highlighting.

### Run-End (DR-2: Hybrid)

- **Victory:** Splash treatment. Stats slam in with energy effects. Highlights animate with impact. Screen shake per reveal.
- **Defeat:** Hologram treatment. Floating holographic display. Stats appear one by one with subtle animation. Calm, contemplative. "Almost unlocked" teases.
- Context-sensitive defeat presentation based on run length (early death = minimal, late death = show what was forming).

Both display: run outcome, nodes cleared, highlights, flux earned, run seed.

### Pause

Overlay on the playing state. `TriggerDesaturation` on the camera to dim the game. Menu options as world-space text. Game entities remain visible but dimmed beneath.

### Loading

Minimal. Progress indicator (simple animated entity) + game logo. Brief — loading is fast.

### Breaker/Bolt Select (Run Setup)

Breaker archetypes displayed as their entity visuals (AttachVisuals with shape/aura/trail). Selection highlights with glow modifiers. Bolt type shown as a preview alongside the breaker.

## What Lives Where

| Concern | Owner |
|---------|-------|
| Per-screen entity spawning + cleanup | `screen/<screen_name>/` |
| VFX primitives used by screens | `rantzsoft_vfx` |
| Screen state definitions | `shared/` (GameState enum) |
| Screen transitions | `screen/transition/` |
| Run-end stats data | `run/` domain (produces stats, screen reads them) |
