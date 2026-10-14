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

`TILE_SIZE` and `ANIM_FPS` live in `src/assets.rs` and are consumed by the
tilemap renderer (`src/tilemap.rs`) and the sprite animator
(`src/animation.rs`).

## Animations

Animation clips are data, authored in Tiled rather than in code. `Main.tmx` has
an **`Animations`** object layer (kept out of the way in the editor and ignored
by the tilemap renderer) whose objects each define one clip. The object's *name*
is the clip name gameplay asks for (`Idle`, `Run`, `Guard`, `Attack1`,
`Attack2`), and its custom properties describe the sheet and playback:

| Property       | Type | Meaning                                   |
| -------------- | ---- | ----------------------------------------- |
| `image`        | file | Sprite sheet, relative to `assets/`        |
| `frame_width`  | int  | Frame width in pixels                      |
| `frame_height` | int  | Frame height in pixels                     |
| `columns`      | int  | Frames per row (sheets are a single row)   |
| `first`        | int  | First frame index of the clip, inclusive   |
| `last`         | int  | Last frame index of the clip, inclusive    |
| `fps`          | int  | Playback rate; falls back to `ANIM_FPS`    |
| `looping`      | bool | `false` marks a one-shot that holds        |

The `Animation` class in `tiny-swords.tiled-project` predefines these members so
Tiled shows typed fields. Add or retune a clip by editing the layer — no
recompile needed; `animation::build_animation_library` parses it at startup.
