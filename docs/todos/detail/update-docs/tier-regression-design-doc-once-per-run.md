# Tier Regression: design doc "once per run" language matches every-other-protocol semantics

## Problems addressed

- `audit/protocols/tier_regression.md` Issue 1 \u2014 Design \u00a7Expected Behavior 2 reads "Can only appear once per run \u2014 whether taken or not" but the impl enforces "once per run after SELECTION" (same as every other protocol). The impl is correct and consistent; the design doc's "whether taken or not" language is wrong.

## Remediation

Open `docs/todos/detail/mod-system-design/protocols/tier_regression.md`.

\u00a7Expected Behaviors 2 \u2014 replace:

> 2. **Can only appear once per run** \u2014 "whether taken or not".

with:

> 2. **Can only appear once per run after being selected** \u2014 matches the convention for every other protocol. If Tier Regression is OFFERED but not SELECTED, it remains eligible for future offerings. Once selected (added to `ActiveProtocols`), it will not appear in subsequent offerings.

\u00a7Edge Cases \u2014 find the line describing re-offering prevention and rewrite to match:

> - If the player rejects Tier Regression at an offering, it returns to the pool of eligible offerings. It does NOT get permanently removed after a single appearance.

No code change. No test change. The impl is correct; this is a design-doc alignment fix.
