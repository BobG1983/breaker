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

## Acknowledged — Low Priority

### Unicode-DFS-2016 warning in cargo deny
- deny.toml allowlist includes Unicode-DFS-2016 but no dep currently uses it.
- This is a harmless pre-approval — keep for forward compatibility.

## Recommendations Deferred

### rand 0.9 → 0.10 (BREAKING)
- Deferred: rand 0.10 is a semver-breaking release. Widespread usage across the codebase
  (bolt, chips, effect, run, shared/rng). Needs a dedicated migration task — not a casual bump.
- Re-evaluate when Bevy ecosystem (bevy_rand etc.) stabilizes on rand 0.10.
