---
name: known-pins
description: Intentional version pins and their rationale — do not flag as outdated
type: project
---

## Intentional Version Pins

### bevy 0.18.1 — all workspace crates
- Pinned to exact minor for Bevy ecosystem coherence.
- bevy_egui 0.39, iyes_progress 0.16 are ecosystem crates paired to bevy 0.18.x.
- Do NOT recommend updating any bevy_* crate independently of the Bevy version.
- All rantzsoft_* crates must stay on the same bevy version as breaker-game.

### iyes_progress 0.16
- Paired to Bevy 0.18. Do not update independently.

### bevy_egui 0.39
- Paired to Bevy 0.18 (bevy_egui 0.39 = bevy 0.18 compatibility release).
- Do not update independently.

### ron 0.12
- Project uses RON for all asset/config files. API-breaking changes in RON would require
  migrating all .ron asset files. Only update with explicit intent to migrate assets.

### proc-macro2 1 (rantzsoft_defaults_derive)
- Explicitly ignored by cargo-machete via [package.metadata.cargo-machete].
- proc-macro2 is a required implicit dep for proc-macro crates even when not directly used.
  This is a known cargo-machete false-positive for proc-macro crates.
