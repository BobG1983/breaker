# Fracture: pin cap-at-4 in design doc; document deterministic fixed-order position selection

## Problems addressed

- `audit/hazards/fracture.md` Issue 5 \u2014 Cap at 4 fragments (matches one of the design's options "2/3/4 per level cap choices"); pin the choice in design.
- `audit/hazards/fracture.md` Issue 6 \u2014 Design \u00a7Edge Cases mentions "seeded `GameRng` for deterministic replay." Impl uses a fixed order (right, left, up, down). Already deterministic; document the choice.

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/fracture.md`.

\u00a7Stacking Behavior (or equivalent) \u2014 pin the cap:

> Fracture caps debris spawns at 4 per death \u2014 one per orthogonal neighbor position. Stack values beyond the cap (stack 5+) do not add more debris per death; they have no effect on spawn count. If the design later wants higher counts (e.g., diagonals for stack 5+), expand the offset set.

\u00a7Edge Cases "position selection" entry \u2014 replace:

> Fracture uses a fixed-order offset list `[(+W, 0), (-W, 0), (0, +H), (0, -H)]` and takes the first `count` entries. Position selection is fully deterministic without RNG. If the count equals 4 (all slots used), order doesn't matter for the final scene. If the count is less than 4, the first `count` slots (right, then left, then up, then down) are used preferentially.
>
> Future tuning could introduce RNG-driven selection (pick `count` of 4 at random) using the project's seeded `GameRng` for determinism. Current impl doesn't need this; the fixed order is sufficient and simpler.

No code change. No test change. Doc-only.
