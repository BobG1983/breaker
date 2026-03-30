# UI & Screens

## HUD Elements

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Node timer | PLACEHOLDER | Critical | High | COVERED | Plain text number in status panel. Color shifts on time fraction. Uses proportional font (should be monospace). |
| Lives display | NONE | Critical | Medium | COVERED (partial) | `LivesCount` tracked but never displayed. |
| Node progress (e.g., 3/8) | NONE | Important | Low | COVERED (partial) | `RunState.node_index` tracked but not displayed. |
| Active chips display | NONE | Low | Low-Med | COVERED (partial) | Left "AUGMENTS" panel exists as empty container. |
| Side panels (structure) | PLACEHOLDER | Low | Low | COVERED | Thin glowing border matching neon dashboard aesthetic. Semi-transparent background. Temperature-following tint. Frames playfield without competing. |
| HUD style (diegetic vs dashboard) | NONE | Critical | Medium | DECISION REQUIRED (DR-1) | No final HUD style chosen. |

## Chip Cards

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Card shape ("cyber chip" outline) | NONE | Important | Medium | COVERED | Cards are plain rectangles. Should be angular/circuit-board-inspired. |
| Common rarity treatment | NONE | Important | Low | COVERED | White/silver glow border, no effects. |
| Rare rarity treatment | NONE | Important | Medium | COVERED | Electric blue glow, subtle pulse. |
| Epic rarity treatment | NONE | Important | Medium-High | COVERED | Magenta glow, shimmer wave animation. |
| Legendary rarity treatment | NONE | Important | High | COVERED | Gold glow (thicker), particle aura, animated energy. |
| Evolution rarity treatment | NONE | Critical | High | COVERED | Prismatic/holographic shifting border, Balatro polychrome shader. |
| Card icon/illustration | NONE | Important | Medium | DECISION REQUIRED (DR-5) | No icons. Cards are text-only. |
| Card selection animation | NONE | Important | Medium | COVERED | Selection tracked but no scale/animation response. |
| Card confirm animation | NONE | Important | Medium | COVERED | No absorption/collapse animation on confirm. |
| Card shatter (timer expired) | NONE | Important | High | COVERED | Timer expiry just ends selection. No card shatter. |
| Timer pressure (50% pulse) | NONE | Important | High | COVERED | No card pulsing at 50%. |
| Timer pressure (25% encroach) | NONE | Important | High | COVERED | No void encroachment at 25%. |
| Timer pressure (10% destabilize) | NONE | Important | High | COVERED | No card flickering at 10%. |

## Screens

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Loading screen | PLACEHOLDER | Low | Low-Med | COVERED | "Loading..." + cyan progress bar. No style treatment. |
| Main menu | PLACEHOLDER | Important | High | COVERED | Orbitron title + menu items. No interactive idle (bolt/breaker playground). |
| Run setup (breaker select) | PLACEHOLDER | Important | High | COVERED | Text-only cards. No archetype visual preview, no color coding. |
| Chip select screen | PLACEHOLDER | Critical | High | COVERED | Functional but plain. No rarity treatments, no card shape, no timer escalation. |
| Pause menu | PLACEHOLDER | Low | Low | COVERED | "PAUSED" + options. No desaturation overlay, no glitch text. |
| Run-end screen | PLACEHOLDER | Important | High | DECISION REQUIRED (DR-2) | Text-only stats + highlights. No animation, no seed display, no "almost unlocked." |
| Run-end "almost unlocked" | NONE | Important | Medium | COVERED | Not implemented. Evolution/achievement teases on defeat. |
| Meta-progression screen | NONE | Low | Low | DEFERRED | `GameState::MetaProgression` exists but no screen. Phase 10 feature. See DR-10 for achievement/discovery UI direction. |

## Typography

| Element | Status | Readability | Juice | Style Guide | Current |
|---------|--------|-------------|-------|-------------|---------|
| Display font (Orbitron-Bold) | PARTIAL | Low | Medium | COVERED | Font loaded. Used for title only. No glitch overlay effects. |
| Body font (Rajdhani-Medium) | PARTIAL | Low | Low | COVERED | Font loaded. Used for menu items only. No scan line treatment. |
| Data font (monospace) | NONE | Critical | Low | COVERED | No monospace font in assets. Timer/seed need monospace for readability. |
| Glitch text shader (scan lines) | NONE | Low | Medium | COVERED | Not implemented. Style guide: scan lines + chromatic split + jitter. |
