# Visual Identity

## Core Aesthetic: Abstract Neon

The game's visual identity is **abstract neon** — no real-world materials, no textures, no physical surfaces. Everything is pure geometry, color fields, and light. The world is digital, not physical. There are no bricks, no paddles, no balls — there are energy constructs, light forms, and glowing fields existing in a void.

This is not retro pixel art. This is not wireframe. This is not photorealistic sci-fi. It is **resolution-agnostic abstract energy** rendered through shaders, basic geometric shapes, HDR bloom, and additive blending.

## Style Pillars

### 1. Light Is the Material

Every visible element is made of light. Nothing is opaque or solid in a physical sense — everything glows, emits, or refracts. The darkness of the void is the canvas; light is the paint. Shadows don't exist because nothing blocks light — everything IS light.

This means: no drop shadows, no ambient occlusion, no material textures. Visual depth comes from glow intensity, bloom radius, and color temperature — not from simulated lighting on surfaces.

### 2. Geometry Over Detail

Shapes are clean, geometric, and immediately readable. A cell is a shape with a glow, not a textured brick. The breaker is an energy form, not a paddle. Visual complexity comes from layering simple shapes with shader effects (bloom, distortion, chromatic aberration), not from high-polygon models or detailed textures.

This serves Pillar 4's mandate: "max spectacle, zero confusion about what matters." Clean geometry reads instantly at 60fps during chaos. Detailed textures do not.

### 3. The Screen Is the Canvas

Full-screen shader effects are first-class visual tools, not post-processing afterthoughts. Gravity wells warp the screen. Shockwaves distort space. Failure pauses time. The entire screen surface — including the background grid — is available for effects to paint on.

Effects apply their visual distortion via their own shaders onto the screen, not by modifying the grid or background directly. The grid is a passive reference surface; effects are active overlays that warp, color-shift, or distort what's beneath them.

### 4. Escalation Is Visible

The game looks different at node 1 than at node 10. The visual escalation mirrors the gameplay escalation (Pillar 1):

- **Temperature**: Cool blues/cyans early in the run shift toward hot magentas/ambers/whites late. The screen literally gets hotter.
- **Complexity**: Early nodes have clean, sparse visuals. Late nodes have more active effects (chip triggers firing VFX, particle trails, stacked visual modifiers on entities). The visual density is earned through build choices.

Temperature communicates risk level. Complexity communicates build power. Together, they make the escalation feel real before a single stat is checked.

### 5. Screenshot-Ready

Every frame should be potentially shareable. The color palette, composition, and spectacle should produce compelling static images, not just compelling motion. This means: strong contrast, vivid colors, clear focal points (bolt and breaker), and enough visual variety that screenshots from different moments look distinct.

This is a commercial requirement, not an artistic preference. Store pages, social media, and streaming thumbnails are how players discover the game. A single compelling screenshot can drive more wishlists than a paragraph of description.

## What This Style Is NOT

- **Not wireframe.** Geometry Wars uses wireframe outlines extensively. We do not. Our shapes are filled with light and energy, not outlined in wire.
- **Not pixel art.** Despite being resolution-agnostic, this is not a pixel aesthetic. Edges are smooth, glows are soft, everything is anti-aliased.
- **Not music-reactive.** The visuals do not react to the music. The music reacts to the gameplay. Visuals react to game state and events directly.
- **Not physically simulated.** No physics-based rendering, no material systems, no light bouncing. Visual effects are authored, not simulated.
- **Not cluttered.** Despite "maximum juice," the bolt and breaker must ALWAYS be trackable. Juice is additive spectacle layered on top of clear gameplay elements, never obscuring them. If a particle storm makes the bolt hard to see, the juice has failed.

## Reference Points

| Game | What We Take | What We Don't |
|------|-------------|---------------|
| Geometry Wars | Brightness, neon-on-black, particle density, grid deformation | Wireframe outlines, retro aesthetic |
| Tetris Effect | Abstract beauty, pacing-reactive atmosphere, emotional color | Music-drives-visuals direction, 3D perspective |
| Rez Infinite | Pacing-reactive intensity, synesthetic feel, abstract digital world | Music-reactive timing, lo-fi geometry |
| Balatro | Card UI treatment (editions, holographic, polychrome) for chip select | 2D hand-drawn style, static presentation |
| Returnal | Particle density during combat, HDR glow, intense visual feedback | 3D perspective, photorealistic materials, AAA scope |
