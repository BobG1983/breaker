# Core Design Principles

> "What if Arkanoid was a roguelite?" — The reflex pressure of Ikaruga meets the build-crafting depth of Slay the Spire, in a breakout game that never lets you breathe.

---

## The Identity: Speed, Juice, Adrenaline

Everything in the game serves one feeling: **relentless forward momentum**. The screen glows, the bolt streaks, cells shatter in neon shrapnel, the timer ticks down, and you're already planning your next move. There is no idle state. There is no safe moment. The game is a sustained adrenaline hit from the moment a run begins until the moment it ends.

This identity is non-negotiable. Every system, every visual, every sound, every mechanic must reinforce it. If something creates dead air, it's wrong. If something slows the player down without adding tension, cut it.

---

## Pillar 1: The Escalation

**Tension only goes up within a run. Relief lives between runs.**

A run is a continuous ramp. Early nodes are manageable — the player is building momentum, learning the layout, assembling their build. Late nodes are white-knuckle — cells are tougher, the timer is shorter, and the build you've constructed is either carrying you or collapsing.

There are no rest stops. Even the upgrade selection screen has a countdown timer. Take too long to decide and you get nothing. The pressure is constant, the intensity only increases, and the only release valve is the run ending — win or lose.

Relief comes *between* runs, in the meta-progression layer. You spend your earned **Flux** on permanent unlocks, reflect on what worked, and prepare for the next attempt. This is the exhale. Then you go back in.

---

## Pillar 2: Build the Broken Build

**The player's goal is to construct a synergy engine that outpaces the difficulty curve.**

This is a "build game" in the tradition of Slay the Spire, Balatro, Binding of Isaac, and Monster Train. The core fantasy is assembling a combination of **Amps**, **Augments**, and **Overclocks** that interact in powerful — sometimes absurd — ways. A piercing bolt that triggers explosions on cell break that chain into more destruction. A breaker so fast it dash-cancels into perfect bumps that send the bolt at devastating speed.

The difficulty scales non-linearly. A competent player with a mediocre build will hit a wall. A skilled player with a broken build will feel unstoppable. The sweet spot is the moment the build "comes online" — when scattered upgrades suddenly click into a machine that chews through nodes. That moment is the high the player chases every run.

Upgrade stacking is intentional. Duplicates compound. There is no cap. If the player commits to a strategy and the run rewards them, the power fantasy should be real.

---

## Pillar 3: Mechanical Floor, Strategic Ceiling

**You need hands to survive. You need a brain to dominate.**

Every player must learn the basics: move the breaker, hit the bolt, clear the cells. The mechanical floor — timing bumps, executing dashes, reading bolt angles — is the entry ticket. Without it, you die.

But mechanical skill alone plateaus. What separates a good player from a great one is strategic depth: reading the upgrade offerings and knowing which build path to commit to, understanding which cells to target first, knowing when a risky dash-cancel is worth the payoff, and recognizing which Overclock triggers synergize with the Amps you've already stacked.

The **perfect bump** is where mechanics and strategy intersect. It's a tight timing window that rewards precise execution with amplified bolt velocity and the ability to cancel a dash — the core high-skill interaction. A novice mashes the bump button. A master times it to the frame, reads the tilt angle, and sends the bolt exactly where it needs to go.

The control surface is deliberately rich: **position** (movement), **angle** (dash/brake tilt), **velocity** (bump timing). Three simultaneous control axes that look simple but take hundreds of hours to master.

---

## Pillar 4: Maximum Juice, Safeguarded Chaos

**If the screen is exploding with neon particle madness, that means you're winning.**

The visual identity is neon cyberpunk — dark backgrounds, glowing electric elements, streaks of light, Tron-like energy. The bolt is a bright streak. The breaker glows and leaves trails on dash. Cells dissolve in showers of particles. Perfect bumps spark. Screen shake punctuates big hits. Chromatic aberration flares on combos. The background pulses with intensity that mirrors the game state.

When a broken build is firing on all cylinders, the screen should look like a rave. This is the visual reward for the power fantasy — you built this chaos, and it's beautiful.

**But the player must never be punished by their own juice.** The bolt and breaker must always remain trackable, even in the most intense moments. Juice enhances the experience; it must never obscure the game state to the point where the player loses a bolt they couldn't see. This means: the bolt and breaker occupy a visual layer that cuts through effects. Particles and screen effects are additive spectacle, not obstacles. If a bolt goes offscreen because the player couldn't track it through the particle storm, the juice has failed.

The rule: **max spectacle, zero confusion about what matters.**

---

## Pillar 5: Pressure, Not Panic

**The game pressures you constantly, but never cheaply.**

The timed upgrade selection is the clearest expression of this. The countdown creates urgency — you can't sit and deliberate forever. But a prepared player who knows the upgrade pool has enough time to read, compare, and choose. The timer punishes *indecision*, not lack of knowledge. Over time, players internalize the upgrade pool and the decision becomes faster, more instinctive. The pressure is psychological: you *feel* rushed even when you have enough time, because the timer is always there.

This principle extends everywhere. The node timer creates urgency but is tuned so that skilled play clears with time to spare. Bolt-lost penalties sting but are recoverable (a life lost, time docked — not instant death). Difficulty spikes are steep but learnable. The game should always feel like it's *almost* too much — riding the edge between flow and overwhelm — without actually being unfair.

**Fair but relentless.** The player should never feel cheated. They should always feel pushed.

---

## Pillar 6: RNG Shapes, Skill Decides

**Randomness creates variety. Execution determines outcomes.**

Every run is seeded and deterministic. The same seed produces the same node sequence, the same upgrade offerings, the same cell layouts. This enables competition ("try my seed") and replayability ("I know I can do better on this seed").

RNG determines the *options* — which upgrades appear, which cell layouts you face, which synergies are available. But the player's skill — both mechanical and strategic — determines the *outcome*. A great player can win a bad seed. A weaker player might lose a generous one. Some seeds are harder than others, but skill always bridges the gap.

The variance is part of the fun. Sometimes RNG hands you a god-tier build and the run feels incredible. Sometimes it forces you to adapt, to find synergies you wouldn't normally pursue. Over many runs, skill is what separates players — not luck.

---

## Pillar 7: Discovery is the Long Game

**The core loop hooks you. Discovery keeps you past the hundredth run.**

The surface game is clear: break cells, build upgrades, survive the timer. But beneath it are layers of hidden depth — secret synergies the game never explains, unlockable content the player doesn't know exists, interactions between upgrades that feel like genuine discoveries.

This is the Isaac influence. The player who has done 10 runs sees a fun action game. The player who has done 100 runs sees a web of interconnected systems with emergent interactions they're still uncovering. The wiki should be essential reading. Community knowledge-sharing should be part of the experience.

Hidden depth is not a "nice to have" — it's critical for longevity. The build-crafting system must support enough emergent complexity that players are still finding new combinations and interactions long after they've mastered the mechanical fundamentals.

---

## Pillar 8: Failure Fuels the Next Run

**How losing feels depends on when and how it happens.**

Failure is not one emotion — it shifts with context:

- **Early death** feels like a quick restart. You barely got going, you see what you did wrong, you're already hitting "try again." Low investment, fast turnaround. *One more run.*

- **Late death** stings with possibility. You can see the build you were constructing, the synergies that were about to come online, the nodes you almost cleared. The near-miss burns — but it's a motivating burn, not a frustrating one. *I was so close.*

- **Spectacular death** transcends failure. Your broken build was firing, the screen was a neon lightshow, you were pulling off combos that felt impossible — and then it ended. But the spectacle was real. The highlight reel was worth it. *That was sick anyway.*

The common thread: **failure must always feel fair, fast, and forward-looking.** The player should never feel stuck, never feel cheated, never feel like the game wasted their time. Every run — even a losing one — teaches something, entertains, and feeds the desire to go again.

---

## The Vocabulary

The game has its own language. This isn't cosmetic — it's identity. The vocabulary reinforces that this isn't "just another breakout game" — it's its own world with its own rules.

See `TERMINOLOGY.md` for the full vocabulary table.

---

## The Litmus Tests

When making any design decision, ask:

1. **Does this increase tension or create dead air?** If it's dead air, cut it or add a timer.
2. **Does this reward player skill or feel random?** If it feels random, the player needs more agency.
3. **Does this create decisions or execute automatically?** If there's no decision, it's not a mechanic — it's a stat.
4. **Does this make builds more interesting?** If it doesn't interact with the upgrade system, it's disconnected from the core loop.
5. **Would a master player use this differently than a novice?** If not, the skill ceiling is too low.
6. **Does this feel fast?** If it doesn't, speed it up until it does.
