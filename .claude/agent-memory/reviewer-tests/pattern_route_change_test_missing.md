---
name: Route change behaviors missing when placed in separate file from new systems
description: When spec puts route-change tests in state/plugin/ but writer-tests only creates new system files, the route-change tests go unwritten
type: feedback
---

When a spec covers both new system behaviors AND a route change in the state plugin (e.g., "AnimateIn was pass-through, now must be message-triggered"), writer-tests tends to focus on the new system files and omit the route-change tests entirely.

Seen in: Wave 1 Bolt Birthing — spec behaviors 22 and 23 (AnimateIn route change) were fully absent. The writer produced 5 new files covering all new systems but made no changes to `state/plugin/tests.rs`.

**Why:** The route-change tests require modifying an existing test file (`state/plugin/tests.rs`) rather than creating a new one. Writer-tests is strong at creating new files but may miss spec behaviors that target existing files when the bulk of the work is creating new ones.

**How to apply:** When reviewing, always cross-check: does the spec reference any existing file for test placement? If yes, verify that file was touched. Behaviors placed in existing files are the most likely to be skipped entirely.
