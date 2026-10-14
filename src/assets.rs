//! Canonical measurements for the Tiny Swords asset pack.
//!
//! These come from the pack itself, so they are the single source of truth for
//! every system that touches terrain or animation timing:
//!
//! - Terrain is authored on a **64x64 pixel** grid.
//! - Animations play at **10 fps**, i.e. one frame every **100 ms**.
//!
//! Levels are authored in Tiled (`assets/Main.tmx`), so the map's tile size
//! should match [`TILE_SIZE`]; the tilemap loader warns when it does not. Unit
//! sprite sheets are a separate scale: they use 192x192 frames (three tiles
//! across), so don't reuse [`TILE_SIZE`] for character animation.

/// Side length of one terrain tile, in pixels.
pub const TILE_SIZE: u32 = 64;

/// Canonical animation playback rate: 10 frames per second, i.e. one frame
/// every 100 ms.
pub const ANIM_FPS: u8 = 10;
