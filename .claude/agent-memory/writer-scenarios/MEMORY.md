# Writer Scenarios Memory

- [RON format conventions](ron_format_conventions.md) — Scenario RON field order, syntax, and common gotchas
- [Effect RON syntax reference](effect_ron_syntax.md) — EffectKind variants with their field names for RON (complete reference)
- [Adversarial patterns by mechanic](adversarial_patterns.md) — Effective adversarial techniques per effect/system domain
- [Invariant substitution for unavailable variants](pattern_invariant_substitution.md) — how to substitute when spec requests non-existent InvariantKind
- [Scenario RON structure and field conventions](pattern_scenario_structure.md) — required/optional fields, stress config, naming, adversarial headers
- [Bevy system extraction anti-pattern](pattern_bevy_system_extraction.md) — never extract FixedUpdate tuples into helper fns; use local let variables instead
