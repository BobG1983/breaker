# RON tuning values — consolidated list of incorrect-to-correct fixes

Single-source consolidation of every RON-only tuning fix across protocols and hazards. Non-RON fixes (code, mechanics, tests) live in per-mechanic detail files under `docs/todos/detail/fix-broken-protocols-and-hazards/`.

Format:

```
Name:             <mechanic>
RON:              <asset path>
Incorrect values: <what's there now>
Correct values:   <what it should be>
```

Per `ron-asset-tests-not-value-pinned.md`: any `tests/ron_asset.rs` that currently pins specific numeric values becomes structural-only after these fixes. Specific numbers live in design-behavior tests that construct configs directly.

---

Name:             Burnout (shockwave)
RON:              `breaker-game/assets/protocols/burnout.protocol.ron`
Incorrect values: shockwave parameters are hardcoded in `burnout/system.rs` (radius, falloff, damage) — not exposed in RON at all
Correct values:   add fields to the RON + `BurnoutConfig`:
                    `shockwave_radius: f32` (move the current hardcoded constant)
                    `shockwave_damage: f32` (move the current hardcoded damage)
                    `shockwave_falloff: f32` (move the current hardcoded falloff coefficient)
                  Existing numeric values preserved — RON values equal the deleted constants.

---

Name:             Cascade (heal values)
RON:              `breaker-game/assets/hazards/cascade.hazard.ron`
Incorrect values: heal amount and per-level scaling wrong per audit `cascade.md` Issue 1
Correct values:   verify against `audit/hazards/cascade.md` §Expected Behavior for the target heal fractions; audit notes specific numbers. Fill in during impl by reading that audit file.

---

Name:             Debt Collector
RON:              `breaker-game/assets/protocols/debt_collector.protocol.ron`
Incorrect values: `stack_per_bump` value does not match design spec
Correct values:   verify target against `audit/protocols/debt_collector.md`; fill in during impl.

---

Name:             Echo Strike
RON:              `breaker-game/assets/protocols/echo_strike.protocol.ron`
Incorrect values: `max_echoes`, `newest_fraction`, `middle_fraction`, `oldest_fraction` do not match design
Correct values:   verify against `audit/protocols/echo_strike.md` §Expected Behaviors; fill in during impl.

---

Name:             Fission (divergence angle)
RON:              `breaker-game/assets/protocols/fission.protocol.ron`
Incorrect values: divergence angle is hardcoded in `fission/system.rs`, not exposed in RON
Correct values:   add `divergence_angle_rad: f32` field to RON + `FissionConfig`. RON value equals the deleted constant.

---

Name:             Iron Curtain (falloff)
RON:              `breaker-game/assets/protocols/iron_curtain.protocol.ron`
Incorrect values: `damage_fraction: 0.25, falloff_start: 0.5` — produces asymmetric falloff that doesn't match design's ABS-symmetric formula
Correct values:   per `audit/protocols/iron_curtain.md`, the design intent is `|x|`-symmetric falloff. RON values may need the same fields renamed/retuned to match the canonical formula. Verify during impl by reading that audit + `iron-curtain-design-doc-abs-symmetric.md`.

---

Name:             Overcharge
RON:              `breaker-game/assets/hazards/overcharge.hazard.ron`
Incorrect values: `base_frac: 0.1, per_level_frac: 0.05` — yields stack-3 × 10-kill ≈ 6.2× speed; design target is 2.84×
Correct values:   `base_frac: 0.05, per_level_frac: 0.03`

Rides with: a field rename in the code (`base_speed_per_kill` → `base_frac`, `per_level_increase_per_kill` → `per_level_frac`) to match the fractional form the math actually uses. Ride the rename with the RON change — single commit.

---

Name:             Haste
RON:              `breaker-game/assets/hazards/haste.hazard.ron`
Incorrect values: `base_percent: 0.1, per_level_percent: 0.05` → yields +0.1% / +0.05% bolt speed boost; design intent is +20% / +10%. Current shipped values are 200× too small.
                  Description is Decay's flavor text (copy-paste error).
Correct values:   `base_percent: 20.0, per_level_percent: 10.0`
                  Description: "Bolts move faster — harder to time your bumps." (verify against design doc at impl time)

---

Name:             Echo Cells
RON:              `breaker-game/assets/hazards/echo_cells.hazard.ron`
Incorrect values: `per_level_multiplier: f32` field name doesn't match the new `HpScaling` enum shape introduced by the per-mechanic fix in `fix-broken-protocols-and-hazards/echo-cells.md`.
Correct values:   replace `per_level_multiplier: 2.0` with `hp_scaling: Doubles` (the new enum variant name). Other fields (`delay_secs`, `base_hp`) unchanged.

Rides with: the `HpScaling` enum introduction (in `echo-cells.md`). RON shape and enum must migrate together — not splittable.

---

## Impl notes

- Each entry is a small, self-contained commit. Name branches `fix/ron-tuning-<mechanic>` and land them in any order.
- Before committing each: grep `tests/ron_asset.rs` for hard-coded numeric assertions on the affected fields and strip those lines (structural validity only, per `ron-asset-tests-not-value-pinned.md`).
- Design-behavior tests (which construct configs directly in code) already pin behaviors against specific numbers; those DON'T change — they're the source of truth for correctness.
- Each RON fix is a BEHAVIOR change. Scenarios that exercise the affected mechanic may need re-recording or re-baselining after the fix lands.

## Out of scope

- Code-driven protocol migrations (Anchor/Deadline/Kickstart/Ricochet): their RONs strip effect trees per TODO #6 — that's a structural RON change, not a value tuning. Covered by #7.
- Structural RON shape changes (field additions, enum migrations): covered by the per-mechanic files they ride with.
- Design-doc alignment for any of the above: separate doc-alignment sweep.
