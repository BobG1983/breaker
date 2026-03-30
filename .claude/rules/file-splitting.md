# File Splitting

Rules for splitting oversized Rust source files. Any agent that splits files must follow these rules.

## Threshold

Any `.rs` file with **400+ total lines** should be split. Lines are lines — blank lines and comments count.

## Strategies

### Strategy A: Test Extraction (test lines > 60% of file)

The dominant case. A file has a small amount of production code and a lot of tests.

**Target — tests under 800 lines:**
```
parent/
  some_system/
    mod.rs          // pub(crate) mod system; #[cfg(test)] mod tests;
    system.rs       // production code only
    tests.rs        // all test code
```

**Target — tests 800+ lines (sub-split by concern):**
```
parent/
  some_system/
    mod.rs          // pub(crate) mod system; #[cfg(test)] mod tests;
    system.rs       // production code only
    tests/
      mod.rs        // mod helpers; mod group_a; mod group_b; ...
      helpers.rs    // shared test setup functions (if any)
      group_a.rs    // tests for behavior A
      group_b.rs    // tests for behavior B
```

Group tests by **behavior** (not alphabetically). Name files after what they test: `damage_tests.rs`, `edge_cases.rs`, `target_dispatch.rs`.

### Strategy B: Concern Separation (mixed production responsibilities)

A file has multiple unrelated production behaviors. Split by concern.

```
parent/
  mixed_file/
    mod.rs              // pub(crate) mod concern_a; pub(crate) mod concern_b; #[cfg(test)] mod tests;
    concern_a.rs        // first responsibility
    concern_b.rs        // second responsibility
    tests/              // (or tests.rs if small enough)
```

### Strategy C: Oversized Test File (already extracted)

A `tests.rs` that is already separate but over 800 lines. Convert to a test directory.

```
parent/
  feature/
    mod.rs          // ← UNCHANGED
    system.rs       // ← UNCHANGED
    tests/
      mod.rs        // mod helpers; mod group_a; mod group_b;
      helpers.rs    // shared test setup (if any)
      group_a.rs
      group_b.rs
```

### mod.rs Violations

If a `mod.rs` contains production code or test code (not just `mod` declarations and `use` re-exports), extract the code into named files. `mod.rs` must be wiring-only.

---

## Split Procedure

Follow this exact sequence. **Do not skip steps. Do not reorder.**

### Case 1: Converting `some_file.rs` to a directory module (Strategy A or B)

#### Step 1: Read the original file completely

Note:
- Where `#[cfg(test)]` begins (divides production from tests)
- All `use` statements at the top
- Which items are `pub` or `pub(crate)` (must be re-exported from mod.rs)
- What the parent module declares (`mod some_file;` in parent's mod.rs or lib.rs)

#### Step 2: Write child files FIRST

Write production code:
```
Write parent/some_file/system.rs
```
Contents: everything BEFORE `#[cfg(test)]`, with all `use` statements the production code needs.

Write test code:
```
Write parent/some_file/tests.rs
```
Contents: everything from `#[cfg(test)]` onwards. Update `use super::*;` to `use super::system::*;` (or specific items). Add any imports from the original header that tests need.

If sub-splitting tests (800+ lines), write each sub-file:
```
Write parent/some_file/tests/mod.rs    // mod group_a; mod group_b; ...
Write parent/some_file/tests/group_a.rs
Write parent/some_file/tests/group_b.rs
```
Each test sub-file needs: `use super::super::system::*;` (up to `some_file/`, then into `system`).

Shared test helpers go in `helpers.rs` with `pub(super)` visibility.

#### Step 3: Write mod.rs LAST (this replaces the original file)

```
Write parent/some_file/mod.rs
```

Contents (and NOTHING else):
```rust
pub(crate) mod system;

#[cfg(test)]
mod tests;
```

Plus re-exports matching what the original file made visible:
```rust
pub(crate) use system::my_system_fn;
pub(crate) use system::MyComponent;
```

#### Step 4: Verify parent module is unchanged

The parent's `mod some_file;` does NOT change. Rust resolves it to either `some_file.rs` or `some_file/mod.rs`. Since we created `some_file/mod.rs`, it works.

**Do NOT edit the parent** unless it had a `use some_file::something` that needs updating (rare — re-exports handle this).

### Case 2: Converting oversized `tests.rs` to a directory (Strategy C)

#### Step 1: Read the test file completely

Note `use` statements, test groups by behavior, shared helpers.

#### Step 2: Write child test files FIRST

```
Write parent/feature/tests/group_a.rs
Write parent/feature/tests/group_b.rs
Write parent/feature/tests/helpers.rs  // if shared helpers exist
```

Each child needs the same `use` statements as the original.

#### Step 3: Write tests/mod.rs LAST (replaces the original tests.rs)

```
Write parent/feature/tests/mod.rs
```

Contents:
```rust
mod helpers;  // if exists
mod group_a;
mod group_b;
```

#### Step 4: Verify parent mod.rs is unchanged

The parent's `#[cfg(test)] mod tests;` does NOT change. Rust resolves `tests.rs` or `tests/mod.rs`.

---

## Import Rules

When code moves deeper, `use super::*` paths change:

| Original location | New location | Path to parent scope |
|---|---|---|
| `some_file.rs` tests using `use super::*` | `some_file/tests.rs` | `use super::system::*;` |
| `some_file.rs` tests using `use super::*` | `some_file/tests/group_a.rs` | `use super::super::system::*;` |
| `tests.rs` using `use super::*` | `tests/group_a.rs` | `use super::super::*;` |

Count directories from the new file to the module containing the items you need. Each directory = one `super::`.

---

## Safety Rules

- **NEVER change function signatures, behavior, or logic** — only move code between files
- **NEVER delete or rename public items** — maintain the same public API via re-exports in mod.rs
- **NEVER modify test assertions or test logic** — tests must behave identically after the split
- **ALWAYS write child files BEFORE mod.rs** — mod.rs replaces the original, so children must exist first
- **ALWAYS re-export** anything the original file made `pub` or `pub(crate)`
- **ALWAYS check** what the parent imports from this file — those imports must still resolve

## Key Rust Rule

You **cannot** have both `foo.rs` and `foo/mod.rs`. When you Write `foo/mod.rs`, the Rust compiler uses the directory module. The orchestrator will clean up the orphaned `foo.rs` after the split.

## `#[cfg(test)]` Scoping

The `#[cfg(test)]` on `mod tests;` in `mod.rs` is sufficient. Sub-modules inside a tests directory do NOT need their own `#[cfg(test)]` attributes.
