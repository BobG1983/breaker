# Phase 6: Audio Foundation

**Goal**: Audio system architecture + placeholder sounds that establish the feel.

## Audio System

- **Audio plugin**: Centralized audio system that responds to game messages
- **Sound categories**: SFX (breaker hit, cell break, perfect bump, chip pickup), ambient (background hum/music layer), UI (menu navigation, selection confirm, timer tick)
- **Per-category volume controls**: SFX, music, UI independently adjustable

## Adaptive Audio (Pillar 9)

Audio must reflect the *state of the run*, not just the current action:

- **Music intensity tied to game state**: Base track layers in additional instruments/tempo as the timer gets low, as the tier number increases, and as the player's build becomes more powerful. A broken build in tier 4 sounds different from a bare build in tier 1.
- **Build-reactive audio**: Specific chip types add audio layers. Piercing bolts have a different impact sound. Shockwaves have a distinct boom. As the build grows, the audio palette grows — each run *sounds* different based on the build assembled.
- **Escalation across nodes**: The audio baseline ramps per tier. Tier 1 is clean and minimal. Tier 3 has more percussion, faster tempo, denser layers. The audio communicates rising stakes without the player reading a number.
- **Boss audio**: Distinct boss music that breaks from the node loop. Signals importance, creates a memorable moment.

## Placeholder Sounds

- Generate or source basic sound effects to validate the system
- Each cell type needs a distinct destruction sound (feedback for learning cell mechanics)
- Perfect bump needs the most satisfying sound in the game — this is the core skill expression
