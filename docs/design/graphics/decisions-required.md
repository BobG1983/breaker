# Decisions Required

Visual design decisions for Phase 5. Most have been resolved; remaining open items noted.

## Resolved

### DR-1: HUD Style — RESOLVED: Diegetic/Integrated

Timer is a bar along the top wall (glow intensity represents time remaining). Lives are orbs near the breaker or along the bottom edge. Node progress is integrated into the playfield frame. No overlaid panels or dashboards — all information lives in the game world.

All HUD data uses monospace typography where numeric (timer, node count). Readability maintained through brightness and positioning rather than UI chrome.

### DR-2: Run-End Screen Style — RESOLVED: Hybrid (Context-Sensitive)

- **Victory**: Splash treatment. Stats slam in with energy effects. Highlights animate with impact. Screen shake per reveal. Celebratory.
- **Defeat**: Hologram treatment. Floating holographic display. Stats appear one by one with subtle animation. Calm, contemplative. Includes "almost unlocked" teases. The "exhale" moment.

Both display: run outcome, nodes cleared, highlight moments, flux earned, notable build milestones, run seed (monospace, prominent, copy-to-clipboard).

Defeat presentation is context-sensitive: early death (1-3) = minimal fanfare; late death (6+) = show what was forming; spectacular death = highlight reel of chaos.

### DR-3: Shield Color — RESOLVED: Patterned White

Pulsing white with a distinctive hexagonal/honeycomb pattern. Distinguished by pattern rather than color — works against any temperature palette and any archetype color. Most future-proof choice.

### DR-4: Memorable Moment Visual Treatments — RESOLVED: Contextual Emphasis

Each highlight type = glitch text label + game element VFX at the relevant location:

| Highlight | Text | Game Element VFX |
|-----------|------|-----------------|
| Close Save | "SAVE." at bottom edge | Barrier flashes |
| Mass Destruction | "OBLITERATE." center-screen | Cell field pulses |
| Combo King | "COMBO." near bolt | Bolt trail intensifies |
| Pinball Wizard | "RICOCHET." at wall | Wall streak effect |
| First Evolution | "EVOLVE." center-screen | Screen glow shift |
| Nail Biter | "CLUTCH." near timer | Timer pulses |

The glitch text shader (scan lines + chromatic split + jitter + punch scale) is the shared infrastructure. The game element VFX is per-highlight.

### DR-5: Chip Card Icons — RESOLVED: Abstract Symbols

Geometric shapes representing effects — circle for AoE, arrow for speed, shield for protection, etc. Consistent with the abstract neon aesthetic. Scales well across 20+ chips without per-chip art. Icons defined as simple geometric compositions, not illustrations.

### DR-6: Grid Line Density — RESOLVED: Configurable (Debug Menu)

Start with medium density. Add a debug menu slider. Tune in-engine once distortion effects exist (step 5k). Grid density is stored in `RenderingDefaults` RON file. Final value determined during implementation.

### DR-7: CRT/Scanline Effect — RESOLVED: Off by Default, Configurable

CRT/scanline overlay exists as a post-processing pass. OFF by default. Configurable in debug menu and eventually in player settings. Default state and intensity stored in `RenderingDefaults` RON file. When a settings menu is added, it writes a user preferences file that overrides `RenderingConfig` after the loading pipeline.

### DR-8: Transition Style Pool Size — RESOLVED: 4 + Extensible

Ship with 4 styles (Flash, Sweep, Glitch, Collapse/Rebuild). System is extensible — adding a new transition means adding an enum variant and defining `rendering/transition/<name>/*`. Add more in Phase 11 polish if playtesting reveals repetition.

### DR-9: Evolution VFX Designs — RESOLVED

All evolution VFX directions reviewed against actual RON behaviors. Key changes from catalog:

**Dropped:** Railgun (merged with Nova Lance — ingredient collision, both from Piercing Shot + Bolt Speed).

**VFX direction corrections:**
- **Nova Lance**: Mechanic needs changing from Shockwave to PiercingBeam (beam fantasy). VFX: thick beam, appears at max width, shrinks over a short duration. Not instant.
- **Railgun**: Dropped (merged into Nova Lance as the beam evolution).
- **Supernova**: NOT a single screen-filling blast. Mechanic is chain reaction (perfect bump → cell destroy → spawn bolts + shockwaves). VFX: base shockwave/bolt-spawn effects with subtle visual marker distinguishing Supernova-triggered effects (brighter ring, extra spark density). Spectacle is emergent from cascade overlap, not a single authored blast.
- **Dead Man's Hand**: Mechanic needs bigger payoff rethink. Current (shockwave + speed boost on bolt loss) is underwhelming. Design deferred to Phase 7.
- **ArcWelder**: VFX matches actual behavior (TetherBeam between bolts, not bolt-to-cells arcs). Enhanced crackling tether, electric corona on both bolts. NOT a Tesla coil.
- **Voltchain**: VFX toned to match mechanic (3 arcs per cell destroy, not screen-filling web). But arcs have LARGE max jumps and louder visual than base chain lightning. Density comes from many cell-destroys in succession.
- **Entropy Engine**: No counter gauge (mechanic has no counter). VFX: prismatic flash on each cell destroy (like Flux), then selected random effect fires. Bolt has prismatic shimmer while active.
- **Phantom Breaker**: Spawns a PhantomBolt (ghost bolt), not a ghost breaker. Future: will have BOTH a Phantom Bolt evolution and a Phantom Breaker evolution (ghost breaker that mirrors movement and bumps).

**VFX directions confirmed as-is:**
- Gravity Well (evolution) — larger/more intense distortion lens
- Split Decision — cell fission effect, energy filaments, prismatic birth trails
- Chain Reaction — recursive shockwaves with escalating light rings (mechanic: cell destroy → small shockwave, shockwave kills → more shockwaves)
- Feedback Loop — three-node triangle charge indicator (mechanic: track 3 perfect bumps → spawn bolt + large shockwave)
- Entropy Engine — prismatic flash per trigger (see correction above)
- FlashStep — breaker teleport on dash (disintegrate → streak → rematerialize)
- Second Wind — invisible wall materialization salvation moment

**Unimplemented evolutions (no RON file, need mechanic + RON in Phase 7):**
- Chain Reaction, Feedback Loop, FlashStep

**Evolutions needing mechanic changes (Phase 7):**
- Nova Lance (Shockwave → PiercingBeam)
- Dead Man's Hand (full rethink)
- Phantom Breaker (split into Phantom Bolt + Phantom Breaker evolutions)

### DR-10: Discovery/Achievement UI — RESOLVED: Visual Language Only

Define the visual treatment vocabulary now; build the screen in Phase 10.

| State | Visual Treatment |
|-------|-----------------|
| Known but locked | Name visible, icon visible, description/reward hidden. Dim glow border, "locked" overlay pattern. |
| Unknown ("????") | Both condition and reward show as "????" placeholder. No glow, minimal styling. Mystery is the aesthetic. |
| Almost unlocked (defeat tease) | Evolution name + abstract symbol icon. Pulsing glow suggesting proximity. "So close" energy. |
| Discovered/Unlocked | Full reveal with rarity-appropriate glow treatment. |

## Architecture Decisions (Phase 5 specific)

### Visual Identity Components — RESOLVED: Separate Components

Each visual property is its own component (`Shape`, `Color`, `AuraType`, `TrailType`, `DamageDisplay`, `DeathEffect`). Entities only get the components that apply to them. Enum types defined in rendering/, attached by owning domain at spawn.

### Render Messages — RESOLVED: Module-Owned Messages

Each VFX module defines its own Bevy message type. Standard systems (not observers) read messages in parallel. A `VfxKind` enum exists for RON data authoring only — the effect/ domain dispatches enum → module message.

### Particle System — RESOLVED: Custom `rantzsoft_particles` Crate

Evaluated bevy_hanabi (macOS pink screen bug #523), bevy_enoki (no additive blending, CPU-only, broken docs), bevy_firework/bevy_sprinkles (3D only), bevy_particle_systems (Bevy 0.14 era). All had disqualifying issues.

Building `rantzsoft_particles` as a new workspace crate. GPU compute shader particle simulation, `Material2d` with additive blending, HDR color support, RON-configurable emitters. Follows rantzsoft_* conventions (game-agnostic, zero game vocabulary).

### Rendering Config — RESOLVED: New RenderingDefaults

New `rendering_defaults.ron` file and `RenderingConfig` resource via the `rantzsoft_defaults` pipeline. Stores CRT toggle, grid density, bloom settings, and other rendering tuning values. rendering/ domain owns it.
