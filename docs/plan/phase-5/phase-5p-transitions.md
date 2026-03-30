# 5p: Transitions

**Goal**: Add new transition styles and random selection, upgrading the existing flash/sweep transitions to match the visual identity.

## What to Build

### 1. Upgrade Existing Transitions

**Flash transition** (PLACEHOLDER):
- Current: Full-screen alpha fade
- Target: Bloom spike + temperature-tinted color (not just white). Brief but impactful.

**Sweep transition** (PLACEHOLDER):
- Current: Full-screen rect sweep with solid color
- Target: Energy beam edge instead of hard color boundary. Beam sweeps with glow and Spark particles trailing the edge.

### 2. Glitch Transition (New)

- Screen corrupts with static/distortion
- Scan line distortion intensifies
- Chromatic aberration splits
- Brief static noise overlay
- Then resolves (In) or blacks out (Out)

### 3. Collapse/Rebuild Transition (New)

- **In (entering node)**: Elements build outward from center point. Grid appears first, then walls, then cells materialize.
- **Out (leaving node)**: Elements collapse inward to center. Cells dissolve, walls retract, grid folds in.

### 4. Random Transition Selection

- System randomly selects one In-style and one Out-style per node transition
- In and Out styles can be different (e.g., Glitch out, Sweep in)
- Selection driven by run seed for deterministic replay
- Pool: Flash, Sweep, Glitch, Collapse/Rebuild (expandable)

### 5. Transition Speed and Pool

All transitions take ~0.3-0.5s. Transitions are fast — Pillar 1 says tension never stops, so transitions should not be rest moments.

Ship with 4 styles. System is **extensible** — adding a new transition means adding an enum variant and defining `rendering/transition/<name>/*`. More can be added in Phase 11 polish if playtesting reveals repetition.

## Dependencies

- **Requires**: 5c (rendering/ absorbed fx/transition code), 5d (post-processing for distortion/chromatic in Glitch transition), 5e (particles for Sweep beam edge), 5f (temperature palette for flash tinting)
- DR-8 resolved: 4 + extensible

## Catalog Elements Addressed

From `catalog/systems.md` (Transitions):
- Flash transition: PLACEHOLDER → bloom spike + temperature tint
- Sweep transition: PLACEHOLDER → energy beam edge
- Glitch transition: NONE → implemented
- Collapse/Rebuild transition: NONE → implemented
- Random transition selection: NONE → implemented

## Verification

- All 4 transition styles work for both In and Out
- Random selection produces different combinations across nodes
- Transitions complete within 0.3-0.5t
- Glitch transition uses screen distortion and chromatic aberration
- Collapse/Rebuild materializes/dematerializes elements smoothly
- All existing tests pass
