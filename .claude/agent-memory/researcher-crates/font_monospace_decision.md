---
name: font_monospace_decision
description: Monospace font evaluation for HUD/UI use — Space Mono recommended over Azeret Mono and Share Tech Mono
type: project
---

Space Mono (OFL 1.1) chosen as the monospace companion for Orbitron-Bold + Rajdhani-Medium in the
brickbreaker UI. Used for: timer numeric readout, run seed display, chip card text, highlight labels.

**Why Space Mono over alternatives:**
- Shares the same Colophon Foundry design language as Space Grotesk; geometric grotesque base
  complements Orbitron's geometry without duplication
- Bold weight available (Regular + Bold + Italics) — critical for dark-background glow readability
- OFL 1.1 — bundle-compatible, no attribution beyond LICENSE file
- Download: https://github.com/googlefonts/spacemono (TTF files in /fonts/)
- Google Fonts: https://fonts.google.com/specimen/Space+Mono
- Weights: Regular, Bold, Regular Italic, Bold Italic

**Rejected:**
- Azeret Mono (OFL 1.1, 9 weights) — richer weight range, but OCR/nineties aesthetic conflicts with
  Orbitron's clean geometry; circular-to-stem discontinuities add visual noise in glow contexts
- Share Tech Mono (OFL 1.1, 1 weight — Regular only) — no bold, ruled out on readability grounds
  for small-size dark-background display

**License:** SIL Open Font License 1.1 — confirmed commercial bundling permitted, attribution via
LICENSE file only, no NC/ND/GPL restrictions.
