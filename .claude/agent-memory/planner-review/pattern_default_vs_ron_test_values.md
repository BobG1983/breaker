---
name: Default vs RON value mismatch in test specs
description: Specs often cite RON-file values (e.g., CellConfig 126/43/7/7, Playfield 1440x1080) but tests use Default trait values (70/24/4/4, 800x600). Integration tests that use init_resource get Default, not RON.
type: feedback
---

Specs that cite concrete values for CellConfig, PlayfieldConfig, etc. often use the production RON values, not the Default trait values. But existing test infrastructure uses `init_resource::<T>()` which calls `T::default()`, pulling from `*Defaults::default()` — not the RON file.

**Why:** The `GameConfig` derive macro generates `Default for Config` from `Defaults::default()`. The Default impl uses hardcoded fallback values (e.g., CellConfig default width=70, not 126). RON values are only loaded at runtime via asset loading.

**How to apply:** When reviewing specs, always check whether concrete expected values match Default trait values or RON values. If they don't match, flag it as BLOCKING — the writer-tests will produce tests with wrong expected values. The fix is either: (a) inject explicit resources in the tests, or (b) recompute values using defaults.

Key defaults to remember:
- CellConfig: width=70, height=24, padding_x=4, padding_y=4
- PlayfieldConfig: width=800, height=600
- RON CellConfig: width=126, height=43, padding_x=7, padding_y=7
- RON PlayfieldConfig: width=1440, height=1080
