# Audio-Visual Sync

How audio and visuals relate to each other. Core principle: **gameplay drives audio, not the reverse.** The music reacts to the game state; the graphics react to game events. Audio and visuals are parallel expressions of the same game state, not coupled to each other.

## Direction of Influence

```
Game State / Events
    ├── drives → Visual Effects (particles, screen FX, color)
    └── drives → Audio (music layers, sound effects)
```

Visuals and audio both read from game state independently. There is no "audio drives visuals" or "visuals drive audio" coupling. This means:
- A shockwave's visual expansion is timed to the game event, not to a beat
- The music's intensity layer is driven by recent action density, not by particle count
- If you muted the game, the visuals would look identical
- If you closed your eyes, the audio would sound identical

## Layered Intensity Music

The music system uses **adaptive layering** — multiple pre-composed layers that fade in/out based on game intensity:

| Intensity Level | Musical Layers | Game State |
|----------------|----------------|------------|
| Low | Ambient pad, minimal rhythm | Early node, few cells remaining, no active combos |
| Medium | Add drums, bass line | Mid-clear, occasional combos, build starting to fire |
| High | Full arrangement, intense rhythm | Active combo chains, shockwaves firing, many effects active |
| Critical | All layers + distortion/urgency elements | Timer critical, last life, final node |

The intensity level is derived from recent game event density (cells destroyed per second, effects triggered per second, timer percentage remaining). It changes smoothly — layers crossfade, not hard-cut.

## Per-Event Sound Effects

Every visual event that matters to the player should have a corresponding sound. The sound and the visual fire at the same time, both triggered by the game event.

| Game Event | Visual | Sound |
|------------|--------|-------|
| Bolt-breaker bump (Perfect) | Gold flash, spark burst, micro-shake | Bright, satisfying impact — high-pitched ring/chime |
| Bolt-breaker bump (Early/Late) | Dim flash, small sparks | Softer impact — duller thud/click |
| Cell destroyed | Adaptive death effect (dissolve/shatter/energy) | Break sound scaled to destruction tier — soft crack vs loud shatter |
| Shockwave | Expanding distortion ring | Bass-heavy boom/whomp, expanding in stereo |
| Chain lightning | Electric arcs | Crackling electric zap, rapid |
| Explosion | Radial burst, screen shake | Deep boom with high-frequency crack |
| Bolt lost | Slow-mo, desaturation | Low-frequency descent tone — "falling" sound |
| Shield absorb | Barrier flash, crack appears | Shield impact — energy absorption sound, bright |
| Shield break (last charge) | Barrier shatter particles | Glass/energy break — shattering sound |
| Gravity well active | Distortion lens, dark void | Low ambient hum — constant while active, pitch shifts with pull strength |
| Evolution trigger | Large screen shake, flash | Dramatic power-up sound — rising tone into impact |
| Timer critical | Timer color shifts to danger | Heartbeat/pulse sound accelerating |
| Node cleared | Freeze-frame, flash | Victory sting — brief triumphant sound |
| Run over (defeat) | Slow-mo, desaturation | Descending tone, silence, then subtle ambient |

## Audio Accessibility

Audio should be functional even without visuals (for accessibility) and visuals should be functional without audio (for muted play). Neither system depends on the other for game-critical information.

## Music Temperature

Just as the visual palette shifts from cool to hot across a run, the music can shift in character:
- **Early nodes**: More ambient, more space, cooler tones
- **Late nodes**: More rhythmic, more dense, warmer/more aggressive tones
- **Boss/final nodes**: Maximum intensity, all layers active

This parallels the visual temperature shift but is independent — both are driven by node progression, not by each other.
