# Reckless Dash: design doc reads `BumpPerformed` (graded bump), not `BoltImpactBreaker` (raw collision)

## Problems addressed

- `audit/protocols/reckless_dash.md` Issue 1 \u2014 Design \u00a7Cross-Domain Dependencies lists `BoltImpactBreaker` (collision/rebound event) but the impl reads `BumpPerformed` (graded game-feature event). These are semantically different: `BoltImpactBreaker` fires on any bolt-breaker collision; `BumpPerformed` fires only when the player executes a timed bump interaction.

## Remediation

Impl is correct. "Risky CATCH" is intentional timing-based gameplay: a non-bumped rebound should not reward the player. Only graded catches count. The design doc's `BoltImpactBreaker` language is wrong.

Open `docs/todos/detail/mod-system-design/protocols/reckless_dash.md` \u00a7Cross-Domain Dependencies. Rewrite the relevant entry to:

> - **breaker domain**: Reads `BumpPerformed` messages. `BumpPerformed` is the graded-bump event (carries grade, bolt, breaker) \u2014 it fires only when the player executes a timed bump, not on passive rebounds. Reckless Dash only considers graded catches.
> - **bolt domain**: Reads `BoltImpactCell` and `BoltLost` messages. Reads bolt's `BoltBaseDamage` for amplified-damage computation.
> - **effect domain**: Manages `EffectStack<DamageBoostConfig>` entries on bolt entities (source-tagged `"protocol:reckless_dash"`) per `damage-amplification-standardization.md` Pattern A \u2014 the `RiskyDamageBoost` component is retired after that migration.

Drop any mention of `BoltImpactBreaker` for this protocol. No code change for Issue 1 specifically.

### Design-doc \u00a7Expected Behaviors clarification

Behavior 1 currently reads "Risky catch grants damage boost." Clarify that "catch" specifically means a graded bump \u2014 passive rebounds do NOT trigger the boost. Add under \u00a7Edge Cases:

> "Catch" means a graded `BumpPerformed` event (the player executed a timed bump during the risky portion of a dash). A bolt that passively rebounds off the breaker without an active bump window does not grant the boost, even if dash progress is in the risky zone.

No test change. Existing tests read `BumpPerformed` and pin the design behavior correctly; they were ahead of the design doc.
