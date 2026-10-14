//! Canonical measurements for the Tiny Swords asset pack.
//!
//! - Terrain is authored on a **64x64 pixel** grid.
//! - Animations default to **10 fps**, i.e. one frame every **100 ms**.
//!
//! Levels *and* animation clips are authored in Tiled (`assets/Main.tmx`). The
//! map's tile size should match [`TILE_SIZE`] (the tilemap loader warns when it
//! does not), and each clip in the `Animations` object layer carries its own
//! frame size, frame range and playback speed. [`ANIM_FPS`] is only the fallback
//! for clips that omit an explicit `fps`. Unit sprite sheets are a separate
//! scale: they use 192x192 frames (three terrain tiles across), so don't reuse
//! [`TILE_SIZE`] for character animation.

/// Side length of one terrain tile, in pixels.
pub const TILE_SIZE: u32 = 64;

/// Default animation playback rate for clips that don't set their own `fps`:
/// 10 frames per second, i.e. one frame every 100 ms.
pub const ANIM_FPS: u8 = 10;
