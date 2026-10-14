# Tiny Words

Thank to [Pixelfrog Assets](https://pixelfrog-assets.itch.io/) for this amazing assets.

## Asset specifications

| Property        | Value                |
| --------------- | -------------------- |
| Tilemap grid    | 64×64 px             |
| Animation speed | 10 fps (100 ms/frame) |

- **Terrain tilesets** (`assets/Terrain/Tileset/Tilemap_color1..5.png`) are a
  9×6 grid of 64px tiles (54 tiles per sheet). `Tilemap_color1.tsx` encodes the
  same layout (`tilewidth=64`, `columns=9`).
- **Flat water** is the single 64px tile `Water Background color.png`; the
  `Water Foam.png` strip is 48 frames of 64px.
- **Unit sprite sheets** use 192×192 frames (three tiles across) — e.g. the
  Warrior idle sheet is 8 frames. Don't reuse the 64px tile size for units.

The canonical numbers live in `src/assets.rs` (`TILE_SIZE`, `ANIM_FPS`) and are
consumed by the sprite animator (`src/animation.rs`) and the tilemap renderer
(`src/tilemap.rs`).
