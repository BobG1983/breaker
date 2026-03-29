---
name: known-findings
description: Accepted / wontfix dependency findings with rationale — skip re-flagging these on future audits
type: project
---

## Wontfix / Accepted Findings

### paste 1.0.15 — RUSTSEC-2024-0436 (unmaintained)
- Advisory: paste is archived/unmaintained
- Path: paste → metal → wgpu-hal → wgpu → bevy_render → bevy 0.18.1
- Rationale: This is a transitive dep several layers deep inside Bevy's render stack. No direct
  project action can fix it. It is NOT a vulnerability (no CVE, no unsound code) — purely a
  maintenance status advisory. Will resolve automatically when Bevy upgrades its wgpu/metal deps.
- Action: None. Re-check after any Bevy version bump.

### bitflags v1.3.2 + v2.11.0 dual versions
- Path for v1: core-graphics → winit; coreaudio-rs → cpal → rodio → bevy_audio
- Path for v2: everywhere else in Bevy
- Rationale: bitflags v1 is required by macOS-specific platform libs (coreaudio-rs, core-graphics).
  These are deep platform transitive deps owned by Bevy/wgpu. Not actionable at project level.
- Impact: Small — bitflags is a lightweight crate, minimal binary/compile overhead.
- Action: None. Re-check after Bevy upgrade.

### objc2 / objc2-app-kit / objc2-foundation dual versions (0.5.x + 0.6.x)
- Rationale: Same root cause as bitflags — macOS platform libs in two generations of objc2 bindings.
  Owned entirely by Bevy's macOS graphics stack. Not actionable at project level.
- Action: None.

### block2 v0.5.1 + v0.6.2 dual versions
- Same root cause as objc2 above.

### core-foundation v0.9.4 + v0.10.1 dual versions
- Same root cause. macOS platform transitive chain.

### hashbrown v0.15.5 + v0.16.1 dual versions
- Two Bevy subcrates using different hashbrown minors. Not actionable.

### read-fonts / skrifa dual versions
- Parley text layout stack split between two font crate generations. Owned by Bevy.

### core-graphics-types v0.1.3 + v0.2.0 dual versions
- macOS graphics transitive chain. Owned by Bevy/wgpu. Not actionable.

### getrandom v0.3.4 + v0.4.2 dual versions
- v0.3.4: rand_core 0.9 → rand 0.9 (project's direct dep)
- v0.4.2: uuid → bevy_animation/bevy_asset/bevy_picking (deep Bevy transitive)
- Rationale: The split is caused by rand 0.9 pinning getrandom 0.3 and Bevy's UUID stack
  pulling getrandom 0.4. Both are lightweight crates. Not actionable without a breaking rand
  upgrade (0.10 would align getrandom to 0.4). Will resolve when rand 0.10 is adopted.
- Action: None. See rand 0.9 → 0.10 deferral below.

### foldhash v0.1.5 + v0.2.0 dual versions
- v0.1.5: hashbrown 0.15 → accesskit_consumer/accesskit_macos, petgraph (Bevy a11y stack)
- v0.2.0: bevy_platform → most of Bevy
- Rationale: Bevy's accessibility/petgraph layer uses an older hashbrown generation with foldhash 0.1.
  Not actionable at project level. Owned by Bevy.
- Action: None.

### itertools v0.13.0 + v0.14.0 dual versions
- v0.13.0: bindgen (build dep for coreaudio-sys → bevy_audio macOS stack)
- v0.14.0: bevy_egui, bevy_math
- Rationale: bindgen is a build-time-only dep for the macOS audio stack. The two versions don't
  coexist in the final binary (build deps compile separately). Not actionable.
- Action: None.

### quick-error v1.2.3 + v2.0.1 dual versions
- v1.2.3: rusty-fork → proptest (dev-only dependency)
- v2.0.1: tiff → image → Bevy rendering
- Rationale: v1 is dev-only (proptest) and v2 is production rendering. No runtime conflict.
  Not actionable — both owned by upstream crates.
- Action: None.

### rustc-hash v1.1.0 + v2.1.1 dual versions
- v1.1.0: cosmic-text, naga, naga_oil, wgpu-core (Bevy render stack)
- v2.1.1: bindgen (build dep for macOS audio stack)
- Rationale: bindgen is build-time only; v1 and v2 are in different compilation contexts.
  Not actionable — all owned by Bevy's render/audio stack.
- Action: None.

### either v1.15.0 dual paths
- One path via itertools 0.13 → bindgen; one path via bevy_animation/bevy_asset and itertools 0.14
- Same version, just referenced from two dep chains. Not a real duplicate (unified by cargo).
- Action: None.

### libc v0.2.183 dual paths
- Same version, multiple paths (clang-sys/bindgen and macOS platform stack).
  Not a real duplicate — unified by cargo.
- Action: None.

### memchr v2.8.0 dual paths
- Same version, multiple paths (regex stack and nom → bindgen).
  Not a real duplicate — unified by cargo.
- Action: None.

### regex / regex-automata / regex-syntax dual paths
- Same versions (regex v1.12.3, regex-automata v0.4.14, regex-syntax v0.8.10) from two paths.
  Not real duplicates — cargo unifies same-version deps.
- Action: None.

## Acknowledged — Low Priority

### Unicode-DFS-2016 warning in cargo deny
- deny.toml allowlist includes Unicode-DFS-2016 but no dep currently uses it.
- This is a harmless pre-approval — keep for forward compatibility.

## Recommendations Deferred

### rand 0.9 → 0.10 (BREAKING)
- Deferred: rand 0.10 is a semver-breaking release. Widespread usage across the codebase
  (bolt, chips, effect, run, shared/rng). Needs a dedicated migration task — not a casual bump.
- Re-evaluate when Bevy ecosystem (bevy_rand etc.) stabilizes on rand 0.10.
- Note: adopting rand 0.10 would also unify the getrandom v0.3/v0.4 split.
