---
name: Normal-to-ImpactSide mapping
description: Wall push-out normals are inverted vs CCD normals for ImpactSide conversion - two separate helper functions needed
type: project
---

Two distinct normal-to-ImpactSide mappings exist in the bolt domain:

**CCD normals** (`ccd_normal_to_impact_side`): Normal points away from struck surface toward bolt origin. Direct mapping:
- NEG_X -> Left, X -> Right, NEG_Y -> Bottom, Y -> Top

**Wall push-out normals** (`wall_normal_to_impact_side`): Normal points away from wall (outward), which is the opposite direction from impact. Inverted mapping:
- X -> Left (pushed right, away from left wall), NEG_X -> Right, Y -> Bottom, NEG_Y -> Top

**Why:** The wall collision system uses a nearest-face approach that gives the push-out direction, while CCD gives the surface normal. These are semantically different normals pointing in opposite directions for the same face.

**How to apply:** Always check which type of normal you have before converting to ImpactSide. Never use one helper for the other's normal type.
