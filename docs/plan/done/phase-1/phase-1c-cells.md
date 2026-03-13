# Phase 1c: Cells

**Goal**: Standard and tough cells with grid layout and destruction.

- Standard cells (1 hit)
- Tough cells (N hits, visual feedback on damage)
- Cell grid layout system driven by data
- Cell destruction effects (placeholder particles/flash)

## What actually shipped

- CellHealth component with multi-hit support and health fraction for visual feedback
- Data-driven grid spawning from config
- Cell destruction via BoltHitCell messages
- HDR color with health-based blue channel range
- Cell width/height as entity components
