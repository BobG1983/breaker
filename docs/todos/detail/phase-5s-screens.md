# 5n: Screens

## Summary

Overhaul all non-gameplay screens to match the game's visual identity. Main menu with interactive idle (breaker + bolt playable in background). Run-end screen with hybrid treatment per DR-2 (victory splash / defeat hologram). Breaker select with per-archetype visual preview. Pause menu with desaturation overlay and GlitchText treatment. Loading screen with styled progress indicator. All screens use world-space entities and the game's shader/material system — not generic Bevy UI.

## Context

All screens currently exist as functional state modules in the `state/` domain (main menu, loading, chip select, pause, etc.) but use placeholder or basic visuals. This phase brings them to the full visual identity: projected neon text, SDF entity rendering, particle effects, and postprocess treatments.

Key architecture change from the LEGACY plan: the old plan referenced `AttachVisuals` messages for breaker preview on the select screen, `ExecuteRecipe` for stats reveal animations on run-end, and crate-owned primitives for transitions. The new plan uses direct entity spawning with EntityGlowMaterial, direct GlitchText spawning for stylized text, direct postprocess trigger messages for screen effects, and direct particle emitter spawning. No recipe indirection. No `AttachVisuals` message.

The old plan placed systems in `screen/`. The state boundary refactor has moved these to `state/` (e.g., `state/menu/main/`, `state/app/loading/`). New systems follow the same `state/` structure.

## What to Build

### 1. Main Menu — Interactive Idle

The main menu IS a gameplay scene — the player can bounce a bolt around casually while considering menu options.

**Playfield setup**:
- Spawn the standard playfield grid background (from 5i wall/background visuals) with walls, but no cells
- Cool temperature palette (early-run, `RunTemperature` at 0.0) — calm, blue-tinted ambiance
- Low particle density — minimal ambient particles, the scene is the "exhale" (Pillar 1)

**Breaker + bolt**:
- Spawn a breaker entity and bolt entity using the standard builders (from 5f, 5g)
- Fully interactive: player can move the breaker and bump the bolt
- No score, no timer, no objectives — just a toy to play with
- Bolt wraps or bounces naturally off walls, never dies (respawns if it somehow exits)

**Menu options**:
- World-space text entities with EntityGlowMaterial glow effects, positioned to not interfere with the play area
- Options: "PLAY" (start run), "SELECT BREAKER" (go to breaker select), "SETTINGS" (future), "QUIT"
- Each option uses Heading-level typography with moderate glitch treatment (GlitchMaterial overlay — scan lines + slight chromatic split)
- Selected option: brighter glow + slight scale increase. Unselected: dimmer.
- Confirm triggers a brief screen flash + transition

**Mood**: Relaxed, inviting. The interactive idle communicates "this is a game about hitting a ball with a paddle" before the player even starts a run.

### 2. Run-End Screen (Hybrid, DR-2)

Two presentations based on outcome:

**Victory — Splash treatment**:
- Dark background (void), no playfield
- Stats slam in with energy effects: each stat line spawns as a GlitchText entity with `PunchScale` + `TriggerScreenShake { tier: Micro }` per reveal
- Staggered reveal: stats appear one at a time with ~0.3s between each, left-to-right or top-to-bottom
- Stats to display:
  - "VICTORY" header (Display typography, full glitch treatment, GlitchText)
  - Nodes cleared: "NODES: X/Y"
  - Highlight moments: list with brief descriptions or icons
  - Flux earned: "FLUX: +XXX"
  - Notable build milestones: evolutions achieved, chip count
  - Run seed: monospace Data typography, prominent placement, copy-to-clipboard affordance (interaction system detects click/keypress on seed, copies to clipboard)
- `TriggerScreenShake { tier: Small }` on the initial "VICTORY" reveal
- `TriggerScreenFlash` white, medium intensity on first reveal
- Celebratory particles: `ParticleEmitter` with `Continuous { rate: 5.0 }` spawning gold/white glow motes drifting upward — ambient celebration

**Defeat — Hologram treatment**:
- Dark background (void)
- Floating holographic display effect: stats appear as dim, slightly transparent entities (alpha ~0.7) with subtle scan-line animation (GlitchMaterial with low-intensity settings)
- Stats appear one by one with subtle fade-in (reverse dissolve over 0.2s per stat), no screen shake — calm, contemplative
- Stats to display: same as victory, but header is "DEFEATED" with cooler/dimmer coloring
- Context-sensitive defeat presentation:
  - **Early death (nodes 1-3)**: Minimal fanfare. Only show nodes cleared, flux earned, and run seed. Quick "TRY AGAIN" option prominent. Low investment = low ceremony.
  - **Late death (nodes 6+)**: Show "ALMOST UNLOCKED" section — evolution names and abstract symbol icons (from 5o) for evolutions the player was 1 chip away from qualifying for. Description hidden, only name + icon with pulsing glow. The "so close" burn that motivates the next run.
  - **Spectacular death (high highlight count)**: Show highlight reel — list the top 3-5 highlights with their GlitchText labels and brief VFX replays (e.g., "OBLITERATE." flashes on screen). "That was sick anyway" energy (Pillar 8).
- Tone is contemplative, not punishing. "That was a run. Here's what happened."

**Shared elements**:
- "CONTINUE" button to return to main menu
- "RETRY" button to start a new run immediately
- Run seed with copy-to-clipboard (Pillar 6 — seed sharing)

### 3. Breaker Select (Run Setup)

Per-archetype visual preview so the player can see what they are choosing:

- Each breaker archetype gets a preview entity spawned with the standard breaker builder (from 5g), including full visual treatment: Shape, Hue, Glow, Aura, Trail
- Previews are positioned in a row (or carousel) at mid-screen
- Selected archetype: full-size, full brightness, aura and trail animating. Slight forward offset (closer to camera).
- Unselected archetypes: smaller, dimmer (GlowIntensity reduced), aura/trail paused or minimal
- Selection change: smooth transition — selected preview scales up and brightens, previously selected scales down and dims. Brief `ParticleEmitter` burst on selection change.
- Archetype name: GlitchText (Heading typography) above the selected preview
- Archetype description: Body typography below the selected preview
- Bolt type preview: small bolt entity beside the selected breaker, showing the archetype's bolt visual (from 5f)
- Confirm: selected breaker flashes + the run begins (transition to first node)

### 4. Pause Menu

Overlays on top of active gameplay:

- `TriggerDesaturation { target_factor: 0.6, duration: 0.2 }` on the game camera — world dims and desaturates beneath the pause overlay
- Game entities remain visible but muted beneath the overlay
- "PAUSED" header: GlitchText with Display typography, full glitch treatment, center-screen
- Menu options as world-space text entities (not UI nodes):
  - "RESUME" — dismiss pause, reverse desaturation
  - "SETTINGS" — future
  - "QUIT TO MENU" — return to main menu
- Selected option: brighter glow + slight scale increase
- Resume: `TriggerDesaturation { target_factor: 0.0, duration: 0.15 }` to smoothly restore color, then unpause game clock

### 5. Loading Screen

Minimal styled loading indicator:

- Void background with subtle grid (from 5i background shader, cool temperature)
- Game logo: GlitchText with Display typography, center-screen, large
- Progress indicator: horizontal bar entity using EntityGlowMaterial, fill level driven by asset loading progress (similar pattern to timer wall gauge but simpler — no danger colors, no pulse)
- Optional: brief text like "LOADING..." in Body typography below the bar, with minimal glitch treatment
- The loading screen should feel like the game's visual identity from the first frame — not a generic loading bar

### 6. Node Transition Effects

Transitions between nodes (not between screens — between gameplay nodes within a run):

- 4 transition styles per DR-8: Flash, Sweep, Glitch, Collapse/Rebuild
- System randomly selects one Out-style and one In-style for each transition (can be different)
- Duration: ~0.3-0.5s per transition (fast — Pillar 1 says tension never stops)

**Flash**: Brief white/color `TriggerScreenFlash` at maximum intensity, instant reveal/hide.

**Sweep**: Energy beam entity (elongated quad with EntityGlowMaterial, HDR >2.0) sweeps horizontally across the screen. Behind the beam: new node revealed (In) or black (Out). Beam moves at constant speed, position updated each frame.

**Glitch**: Screen corrupts with `TriggerChromaticAberration` at high intensity + `TriggerDesaturation` flickering + GlitchMaterial full-screen overlay (spawn a full-viewport quad with GlitchMaterial at extreme settings). After ~0.2s, resolves to new node (In) or black (Out).

**Collapse/Rebuild**: Elements (cells, walls) scale toward/away from center point. Out: all entities scale to 0 at center over 0.3s. In: entities spawn at center and expand to final positions over 0.3s. Uses `PunchScale` or per-frame Transform scale animation.

Transition system lives in the state domain (wherever node-to-node transitions are managed).

### 7. Screen Transition Infrastructure

Generic screen transition system for transitions between states (menu -> game, game -> run-end, etc.):

- Simple fade-to-black and fade-from-black using `TriggerDesaturation` + alpha overlay
- Duration: ~0.2-0.3s
- Can be extended with more elaborate transitions later (Phase 11 polish)

## What NOT to Do

- Do NOT implement a settings menu — that is a future feature. Add the menu option but have it do nothing or show a "COMING SOON" placeholder.
- Do NOT implement the discovery/achievement screen — that is Phase 10 per DR-10
- Do NOT implement the chip select screen layout/logic — that is 5o (chip cards). This phase handles the other screens.
- Do NOT use Bevy UI nodes (Node, Button, Interaction) for screen layouts — use world-space entities. Exception: if cursor-to-world input for menu selection proves impractical, Bevy UI may be used for input handling only, with visual rendering still in world-space.
- Do NOT implement elaborate animations that delay gameplay start — transitions and reveals should be fast (Pillar 1)

## Dependencies

- **Requires**: 5e (visuals domain — EntityGlowMaterial, GlitchMaterial, HolographicMaterial, PunchScale, FadeOut, Shape, Hue), 5o (chip cards — chip card visual patterns referenced for consistency; abstract symbol icons for "almost unlocked" evolution teases on defeat screen)
- **Enhanced by**: 5c (rantzsoft_particles2d — celebration particles on victory, selection bursts), 5d (rantzsoft_postprocess — desaturation for pause, screen flash for transitions, chromatic aberration for glitch transition), 5f (bolt visuals — bolt preview on breaker select, interactive bolt on main menu), 5g (breaker visuals — breaker preview on select screen, interactive breaker on main menu), 5j (screen effects — screen shake for victory stat reveals, slow-mo infrastructure), 5m (GlitchText — for all stylized text on screens), 5n (HUD — monospace font for run seed display)
- **Independent of**: 5k (bump/failure VFX), 5l (combat VFX), 5q-5t (evolution VFX)

## Verification

- **Main menu**: Playfield visible with grid and walls. Breaker and bolt interactive — player can move breaker and bump bolt. Menu options visible with glow treatment. Cool palette ambiance.
- **Run-end (victory)**: "VICTORY" header with full glitch treatment. Stats slam in one at a time with screen shake per reveal. Celebration particles. Run seed visible in monospace, copy-to-clipboard works.
- **Run-end (defeat)**: "DEFEATED" header, hologram treatment (dim, transparent, scan lines). Stats fade in one at a time, no shake. Contemplative tone.
- **Run-end (defeat, late death)**: "ALMOST UNLOCKED" section shows evolution names + icons with pulsing glow. Descriptions hidden.
- **Run-end (defeat, spectacular)**: Highlight reel shows top highlights with GlitchText labels.
- **Run-end (defeat, early death)**: Minimal — just nodes, flux, seed, and prominent "TRY AGAIN".
- **Breaker select**: Archetype previews show full visual treatment (shape, color, glow, aura, trail). Selected archetype is larger/brighter. Bolt preview visible. Selection transitions are smooth.
- **Pause**: Game world desaturates beneath overlay. "PAUSED" GlitchText visible. Menu options respond to input. Resume restores color smoothly.
- **Loading**: Game logo visible with glitch treatment. Progress bar fills correctly. Visual identity present from first frame.
- **Node transitions**: All 4 styles work (Flash, Sweep, Glitch, Collapse/Rebuild). Random selection varies transitions across nodes. Each transition is 0.3-0.5s. In and Out styles can differ.
- All screen entity lifecycles are clean: spawn on enter, despawn on exit, no leaked entities
- All existing tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status: NEEDS DETAIL
