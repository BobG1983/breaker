# TetherBeam

Bolts connected by crackling neon beams that damage everything they intersect. Standard mode spawns two bolts with a connecting beam; chain mode tethers all active bolts in sequence.

For technical details (config struct, standard vs chain mode mechanics, fire behavior), see `docs/architecture/effects/effect_reference.md`.

## VFX

### Standard mode
- Crackling electric energy along the tether beam
- Elasticity visual — stretches when bolts far apart, slackens when close
- Animated energy flowing along beam (brightness traveling end to end)
- Flash + sparks when tether snaps

### Chain mode (ArcWelder evolution)
- Same crackling energy per beam segment
- Electric corona on all connected bolts
- Chain forms a visible electric web across the playfield when many bolts are active
