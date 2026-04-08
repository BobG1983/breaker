# Runner Scenarios Agent Memory

- [entered_playing reset pattern](stable_entered_playing_reset.md) — must reset to false on run restart; absence causes BreakerCountReasonable false positives during teardown gaps
- [Layout name case sensitivity](stable_layout_name_casing.md) — boss_arena vs BossArena mismatch pattern
- [entropy_engine_stress birthing regression](stable_entropy_engine_stress_birthing.md) — .birthed() on SpawnBolts changed accumulation dynamics; max_bolt_count:12 no longer reached with seed 4242
