# 5l: Bump Grade & Failure State VFX

**Goal**: Replace floating text feedback with visual-only feedback for bump grades and failure states. All feedback communicates through light, motion, and screen effects — no text.

## What to Build

### 1. Bump Grade Feedback

Replace floating text ("PERFECT", "EARLY", "LATE", "WHIFF") with visual-only feedback:

| Grade | Breaker Response | Screen Response | Particle Response |
|-------|-----------------|-----------------|-------------------|
| Perfect | Bright gold/white flash. VFX burst from impact. Full archetype glow intensifies. | Micro-shake. HDR bloom spike at impact. | Spark burst — bright, vivid, numerous |
| Early | Dim archetype-color flash. Subtle VFX at impact. | None | Small Spark burst — fewer, dimmer |
| Late | Dim archetype-color flash. Minimal VFX. | None | Minimal Sparks — barely visible |
| Whiff | No visual response. | None | None. Silence IS feedback. |

Translation layer: existing `BumpOccurred` gameplay message → module-owned `PlayBumpFeedbackVfx { grade, position, archetype_color }` message.

### 2. Bolt Lost VFX

Replace "BOLT LOST" floating text:
- Slow-mo (~0.3s at 30% speed) via `TriggerSlowMotion` from 5k
- Bolt leaves bright exit streak as it falls below playfield
- Brief screen desaturation (from 5d/5k)
- Void below "swallows" the bolt (bolt fades as it exits)
- Snap back to full speed, new bolt spawns with flash (bolt spawn moment from 5g)

### 3. Shield Absorption VFX

When shield absorbs a bolt-loss:
- Bottom barrier flashes bright (from 5j shield visual)
- Cracks appear on barrier
- Bolt bounces back
- No slow-mo — the save is instant and satisfying
- Shield charge indicator updates (if visible)

### 4. Life Lost VFX

When a life is lost:
- Longer slow-mo (~0.5t at 20% speed)
- Red-orange screen flash at edges (danger vignette pulse from 5k)
- Life indicator visibly depletes
- More dramatic than bolt-lost, less dramatic than run-over

### 5. Run Won VFX

When final node is cleared:
- Brief freeze-frame at moment of last cell destruction
- Screen flash (white)
- Transition to run-end screen with spectacle

### 6. Run Over (Defeat) VFX

When final life lost or timer expires:
- Extended slow-mo
- Screen desaturates fully to near-monochrome
- Final bolt trail fades
- All particles slow and dim
- Transition to run-end screen
- Tone: contemplative, not punishing (Pillar 8)

### 7. Time Expired VFX

When the node timer hits zero:
- Timer display shatters into Shard particles
- Red-orange pulse radiates from timer across screen edges
- Dark wave sweeps downward
- Desaturates from edges inward

### 8. Remove Floating Text

Remove all existing floating text feedback:
- "PERFECT", "EARLY", "LATE", "WHIFF" text popups
- "BOLT LOST" text
- Replace with the visual-only systems above

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing: desaturation, flash), 5e (particles: Spark, Shard), 5g (bolt visuals for exit streak, spawn flash), 5j (shield barrier for absorption VFX), 5k (screen effects: shake, slow-mo, vignette, flash)

## What This Step Builds

- Visual-only bump grade feedback (Perfect: gold flash + sparks + shake; Early/Late: dim flash + few sparks; Whiff: nothing)
- Bolt lost VFX (slow-mo + exit streak + desaturation)
- Shield absorption VFX (barrier flash + cracks, no slow-mo)
- Life lost VFX (longer slow-mo + danger vignette)
- Run won VFX (freeze-frame + white flash)
- Run over VFX (extended slow-mo + full desaturation)
- Time expired VFX (timer shatter + red-orange pulse + dark wave)
- Remove all floating text feedback (PERFECT, EARLY, LATE, WHIFF, BOLT LOST)

## Verification

- No floating text remains for bump grades or failure states
- Perfect bump produces visible gold flash + sparks + micro-shake
- Whiff produces zero visual feedback
- Bolt lost triggers slow-mo + streak + desaturation
- Shield absorption triggers barrier flash without slow-mo
- Run end states produce appropriate VFX
- All existing tests pass
