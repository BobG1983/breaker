# Retire SFX from every protocol and hazard design doc (defer to Phase 6 audio)

## Problems addressed

- `audit/protocols/conductor.md` Issue 4 — Conductor SFX hook pending; audio domain is a Phase-0 stub.
- (This file absorbs the equivalent SFX-deferral issue from every other protocol and hazard as the interrogation reaches each file. Currently only Conductor's design doc mentions SFX; any additional hits will be appended to §Problems addressed.)

## Remediation

No protocol or hazard ships SFX in the current project phase. Audio work is consolidated under TODO.md item 30 (Phase 6: Audio foundation). Until that phase lands, protocol and hazard design docs should not specify audio behavior, and no protocol/hazard code should call into the audio domain.

Sweep `docs/todos/detail/mod-system-design/protocols/*.md` and `docs/todos/detail/mod-system-design/hazards/*.md`. In each file:

- Remove any \u00a7SFX section or line item that specifies audio behavior.
- If the file currently lists audio parameters (sound IDs, volumes, timings), delete them.
- Add a one-line entry in a \u00a7Deferred concerns section at the end of the doc: "SFX is deferred to Phase 6 (audio foundation, TODO.md item 30). No audio hooks in this protocol/hazard until Phase 6 lands."

Today only `conductor.md` has an SFX mention (confirmed by grep of `mod-system-design/protocols` and `mod-system-design/hazards`). If future audits surface additional files, they get the same treatment in the same commit.

No code changes required \u2014 protocols and hazards currently don't emit audio messages (Conductor's SFX hook was spec'd but never implemented per the conductor audit). The remediation is purely design-doc alignment so that "what the design doc says" matches "what the code does" through Phase 5.

When Phase 6 begins, this remediation becomes obsolete: at that point each protocol/hazard revisits its \u00a7Deferred concerns entry, re-adds a concrete \u00a7SFX section with specific trigger + message + sound-design parameters, and the implementation emits `PlaySfx { id: ... }` messages consumed by the audio domain. That work is out of scope here.
