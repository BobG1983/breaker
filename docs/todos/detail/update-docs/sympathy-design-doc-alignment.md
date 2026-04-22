# Sympathy — canonical design doc system + pipeline position

## Target file

`docs/design/hazards/sympathy.md` (promoted during this sweep).

## What the target doc must say

§ Systems:

> **`sympathy_heal_adjacent`** (schedule: `FixedUpdate`, set `DeathPipelineSystems::EmitHeal`, ordered after the `MutateDamage` chain has run).
>
> Reads `DamageDealt<Cell>` messages AFTER the `MutateDamage` chain has transformed the damage (so Diffusion-reduced or Tether-redirected values, if any). For each damaged cell, BFS outward through adjacent live cells and emit `HealDealt<Cell>` messages with geometric ring attenuation. The heal is applied same-tick via `apply_heal::<Cell>`.

§ Pipeline position:

> Sympathy is a **post-apply reactor** — it reads `DamageDealt<Cell>` downstream of the damage-mutator chain and emits `HealDealt<Cell>`. Contrast with `MessageMutator<DamageDealt<Cell>>`-style hazards (Diffusion, Tether) that TRANSFORM the damage message inside `DeathPipelineSystems::MutateDamage` before apply.
>
> Both classes live in `mutators/hazards/` post-TODO #2; the distinction is pipeline position, not domain.

## What the target doc must NOT say

- Do not claim "the cells domain's `apply_damage::<Cell>` reads `SympathyConfig`" — Sympathy lives in `mutators/hazards/sympathy/` and consumes messages, not configs from another domain.
- Do not describe Sympathy as a pre-apply damage mutator — it is a reactor.
- Do not frame the mutator-vs-reactor distinction as a domain-split (`hazard/` vs `cells/`) — both classes now live in `mutators/` (post-TODO #2).

## Why

Sympathy's mechanical identity — heals neighbours in response to damage — is fundamentally a post-apply reaction, not a damage transformation. TODO #2 consolidates both mutator-style and reactor-style hazards into `mutators/`, eliminating the prior domain split but preserving the pipeline-position distinction.
