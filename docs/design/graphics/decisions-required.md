# Decisions Required

Open visual design questions that need prototyping, visual exploration, or generative AI mockups before they can be resolved.

## High Priority (Blocks Other Decisions)

### DR-1: HUD Style — Diegetic vs Neon Dashboard

**Context**: The HUD displays timer, lives, node progress, and optionally active chips during gameplay. Two candidate styles exist.

**Option A: Diegetic/Integrated**
- Timer is a bar along the top wall (or built into the wall glow intensity)
- Lives are orbs near the breaker or along the bottom edge
- Node progress is integrated into the playfield frame
- Pro: Maximum immersion. Nothing "overlaid" on the game.
- Con: Harder to read at a glance. Competes with gameplay elements for visual space.

**Option B: Neon Dashboard**
- HUD elements are holographic neon readouts floating above/beside the playfield
- Styled as projected displays matching the game's typography (glitched/stylized)
- Pro: Instantly readable. Clear separation between game and info. Cohesive with sci-fi aesthetic.
- Con: Less immersive. Takes screen real estate.

**What's needed**: Generative AI mockups of both styles overlaid on a gameplay screenshot. Compare readability, immersion, and how each handles the "critical timer" state.

### DR-2: Run-End Screen Style — Hologram vs Splash

**Context**: The run-end screen shows highlights ("Every Run Tells a Story"). Two candidate styles exist.

**Option A: Scorecard Hologram**
- Calm, debriefing feel. Stats projected as a holographic floating display.
- Highlights appear one by one with subtle animation.
- Pro: Fits the sci-fi aesthetic. The "exhale" moment (Pillar 1). Feels like reviewing a mission log.
- Con: Might feel anticlimactic after an intense run.

**Option B: Victory/Defeat Splash**
- Dramatic reveal. Stats slam in with energy effects. Highlights animate with impact.
- Pro: The run-end is itself a spectacle — the final "moment." Emotionally charged.
- Con: Might feel exhausting after an already-intense run. Might undermine the relief of Pillar 1's "exhale."

**Option C: Hybrid** (potential compromise)
- Victory gets the splash treatment (celebratory). Defeat gets the hologram treatment (contemplative).
- Different emotions for different outcomes.

**What's needed**: Generative AI mockups of both styles, plus the hybrid. Test whether the hologram feels anticlimactic and whether the splash feels exhausting.

## Medium Priority (Needed Before Catalog)

### DR-3: Shield Color

**Context**: Shields (bolt-loss protection) appear as an energy barrier along the bottom wall. The shield color must be:
- Instantly distinguishable from the bolt (white/warm glow)
- Instantly distinguishable from the breaker's archetype color
- Readable against any temperature palette (cool and hot)

**Candidates**:
- Green (distinct from all archetypes, but conflicts with Regen cell color)
- Bright cyan (distinct from warm archetypes, but close to Aegis blue)
- Amber/gold (high visibility, but conflicts with Chrono archetype and danger color)
- Unique: pulsing white with a distinctive pattern (hexagonal grid, honeycomb) that distinguishes it by pattern rather than color

**What's needed**: Test each candidate against all three archetype color schemes and both temperature extremes (cool and hot palette).

### DR-4: Memorable Moment Visual Treatments

**Context**: In-game highlight popups (Close Save, Mass Destruction, Combo King, Pinball Wizard, First Evolution, Nail Biter) are currently text-based. They need to be redesigned as visual-only effects.

Each highlight type needs a distinctive visual treatment that communicates what happened without text:

| Highlight | Current (text) | Needs Visual Equivalent |
|-----------|---------------|------------------------|
| Close Save | Text popup "Close Save!" | Bottom-edge flash? Shield shimmer? Near-miss streak? |
| Mass Destruction | Text popup "Mass Destruction!" | Screen-wide particle burst? Brief white-hot flash? |
| Combo King | Text popup "Combo King!" | Bolt trail intensifies? Combo streak visual? |
| Pinball Wizard | Text popup "Pinball Wizard!" | Wall-bounce streak effect? Ricochet trails? |
| First Evolution | Text popup "First Evolution!" | Evolution-specific flash? Build-glow change? |
| Nail Biter | Text popup "Nail Biter!" | Timer pulse? Edge-of-screen urgency flash? |

**What's needed**: Design a consistent visual language for highlight moments. Options: (A) each gets a bespoke visual, (B) all share a common "moment" frame (brief slow-mo + distinctive icon flash) with per-type color/shape, (C) highlight type determines which game element gets the visual emphasis (Close Save emphasizes the barrier, Combo King emphasizes the bolt).

### DR-5: Chip Card Icons/Illustrations

**Context**: Each chip card needs a visual icon/illustration representing its effect. With 20+ chips, plus rarity variants and evolutions, this is a significant art requirement.

**Options**:
- Abstract symbols (geometric shapes representing effects — circle for AoE, arrow for speed, etc.)
- Miniature effect previews (tiny version of the actual VFX the chip produces)
- Unique illustrations per chip (highest quality, most expensive to produce)
- Combination (abstract symbol for common, miniature preview for rare+, unique for evolutions)

**What's needed**: Decide on the icon strategy before the catalog. This affects how many unique graphics need to be created vs generated from the effect system.

## Lower Priority (Can Be Resolved During Implementation)

### DR-6: Grid Line Density and Spacing

**Context**: The background grid's line density affects how visible distortion effects are (more lines = more visible warping) but also affects visual cleanliness (fewer lines = cleaner).

**What's needed**: Test 3-4 grid densities in-engine against a gravity well distortion to find the sweet spot.

### DR-7: CRT/Scanline Effect

**Context**: The debug menu should allow toggling a CRT/scanline overlay for the retro-digital feel. Questions: default on or off? How intense? Applied to the whole screen or just UI text?

**What's needed**: In-engine testing with CRT shader at various intensities. This is a tuning question, not a design question.

### DR-8: Transition Style Pool Size

**Context**: Four transition styles are defined (Flash, Sweep, Glitch, Collapse/Rebuild). Are four enough variety, or should more be designed?

**What's needed**: Implement the four, playtest across multiple runs, see if transitions feel repetitive. Add more only if needed.

### DR-9: Evolution VFX Designs

**Context**: Each of the 8 evolutions needs bespoke VFX. These are the most visually ambitious effects in the game. Each needs individual design attention.

**Evolutions needing VFX design**:
1. Nova Lance — massive beam
2. Voltchain — branching lightning web
3. Phantom Breaker — ghost breaker mirror
4. Supernova — screen-filling explosion
5. Dead Man's Hand — synchronized bolt pulse
6. Railgun — instantaneous thin beam
7. Gravity Well — distortion void (partially designed)
8. Second Wind — invisible wall materialization

**What's needed**: Individual VFX design documents or concept sketches for each evolution. These are the visual crown jewels and deserve dedicated design time.
