# Phase 1b: Bolt

**Goal**: Custom physics bolt with responsive collision and breaker reflection.

- Custom physics: velocity-based movement, no engine physics
- Collision detection: breaker, cells, walls, ceiling
- **Breaker reflection model**: Direction entirely overwritten on breaker contact based on:
  - Hit position on breaker (left/right of center → angle)
  - Breaker tilt state (dashing/braking tilt modifies the effective surface angle)
  - Bump grade (perfect/early/late/none → velocity magnitude)
  - No incoming angle carryover. No perfectly vertical or horizontal reflections.
- Speed management: base speed, speed caps, speed modifications from upgrades and bump grade
- Bolt-lost detection (falls below breaker)

## What actually shipped

Beyond the plan:
- Swept CCD (continuous collision detection) with ray-vs-expanded-AABB
- Multi-bounce per frame with MAX_BOUNCES cap
- Bolt serve mechanic (hover on first node, launch on bump)
- Bolt-lost visual feedback (fading text)
- Bolt respawn at base_speed
- Minimum angle enforcement (never perfectly horizontal)
- Base speed floor clamp on weak bumps
- Wall domain extraction (invisible boundary entities for CCD)
