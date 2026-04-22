# Iron Curtain: design doc cleanup \u2014 keep abs-symmetric falloff, delete "upward only" prose

## Problems addressed

- `audit/protocols/iron_curtain.md` Issue 2 \u2014 Design \u00a7Edge Cases prose says "wave spreads upward only" but the design's own \u00a7Damage Falloff Formula uses `(cell_position.y - breaker_position.y).abs()`, and the impl + tests pin the abs-symmetric behavior. The contradiction is within the design doc itself; the code and tests are consistent.

## Remediation

The design's falloff formula wins. Abs-symmetric is the canonical behavior.

Open `docs/todos/detail/mod-system-design/protocols/iron_curtain.md`. Under \u00a7Edge Cases, delete the line that reads "Cells behind the breaker (below it) are not damaged \u2014 wave spreads upward only." (or the equivalent wording \u2014 whichever line captures the upward-only claim). Replace with:

> Cells at distance `|cell.y - breaker.y|` within the falloff radius are damaged symmetrically. In practice, cells rarely spawn below the breaker, but the wave does not explicitly mask them \u2014 if a cell exists below, it receives damage consistent with the symmetric falloff formula.

No code change. No test change. The `cells_below_breaker_are_damaged_by_abs_symmetric_falloff` test at `tests/on_bolt_lost.rs:294-313` already pins the intended behavior and stays as-is.
