//! Tiled map loading and rendering.
//!
//! [`load_map`] parses the map in `assets/Main.tmx` with the [`tiled`] crate
//! and stores it in the [`LevelMap`] resource. [`spawn_tiled_map`] then expands
//! every visible tile layer into one [`Sprite`] per tile, so a level is authored
//! entirely in the Tiled editor: no per-tile code and no rebuild to change the
//! layout.
//!
//! Tiles are placed one-per-entity at their native size, keeping the grid
//! pixel-aligned with the art. The map's own tile size drives placement, and
//! the parsed [`tiled::Map`] is kept around so gameplay systems (collisions,
//! spawn points, ...) can read it later.

use std::env;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use tiled::{ChunkData, LayerTile, TileLayer};

use crate::assets::TILE_SIZE;

/// Bevy's default asset folder name (see `AssetPlugin::file_path`).
const ASSET_DIR: &str = "assets";

/// The Tiled map to load, inside [`ASSET_DIR`].
const MAP_FILE: &str = "Main.tmx";

/// The parsed Tiled map, shared with gameplay systems.
#[derive(Resource)]
pub struct LevelMap(pub tiled::Map);

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_map, spawn_tiled_map).chain());
    }
}

/// Directory the `assets/` root is resolved against.
///
/// Mirrors `bevy_asset`'s own logic (`BEVY_ASSET_ROOT`, then
/// `CARGO_MANIFEST_DIR`, then the executable's directory) so the map loader and
/// the [`AssetServer`] always agree on where assets live.
fn base_path() -> PathBuf {
    if let Ok(root) = env::var("BEVY_ASSET_ROOT") {
        PathBuf::from(root)
    } else if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest)
    } else {
        env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .unwrap_or_default()
    }
}

/// Absolute path to the `assets/` directory.
fn assets_dir() -> PathBuf {
    base_path().join(ASSET_DIR)
}

/// Parses the Tiled map into the [`LevelMap`] resource.
fn load_map(mut commands: Commands) {
    let map_path = assets_dir().join(MAP_FILE);
    match tiled::Loader::new().load_tmx_map(&map_path) {
        Ok(map) => {
            info!(
                "loaded {}: {}x{} tiles of {}x{} px, {} layer(s), {} tileset(s)",
                map_path.display(),
                map.width,
                map.height,
                map.tile_width,
                map.tile_height,
                map.layers().len(),
                map.tilesets().len(),
            );
            commands.insert_resource(LevelMap(map));
        }
        Err(error) => error!("failed to load {}: {error}", map_path.display()),
    }
}

/// Image and atlas handles for one single-image tileset.
struct TilesetAssets {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

/// Expands every visible tile layer of [`LevelMap`] into one sprite per tile.
fn spawn_tiled_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    level: Option<Res<LevelMap>>,
) {
    let Some(level) = level else {
        return;
    };
    let map = &level.0;

    if map.tile_width != TILE_SIZE || map.tile_height != TILE_SIZE {
        warn!(
            "map grid is {}x{} px, expected {TILE_SIZE}px (see assets::TILE_SIZE)",
            map.tile_width, map.tile_height,
        );
    }

    // The map is centered on the world origin, matching the player spawn point.
    let tile_width = map.tile_width as f32;
    let tile_height = map.tile_height as f32;
    let origin_x = -(map.width as f32 - 1.0) * tile_width / 2.0;
    let origin_y = (map.height as f32 - 1.0) * tile_height / 2.0;

    // Layers are listed bottom-first, so later layers draw on top. The whole
    // stack is kept below the player (spawned at z = 1.0) by placing the top
    // layer at z = -1.0 and stacking downward.
    let layer_count = map.layers().len() as f32;

    let tilesets = build_tileset_assets(map, &asset_server, &mut layouts, &assets_dir());

    let origin = Vec2::new(origin_x, origin_y);
    let tile_size = Vec2::new(tile_width, tile_height);

    for (depth, layer) in map.layers().enumerate() {
        if !layer.visible {
            continue;
        }
        let Some(tile_layer) = layer.as_tile_layer() else {
            continue;
        };

        let z = -(layer_count - depth as f32);

        // Finite layers are a fixed grid; infinite layers store tiles in 16x16
        // chunks with no overall bounds, so walk the chunks that exist.
        match tile_layer {
            TileLayer::Finite(finite) => {
                for y in 0..finite.height() as i32 {
                    for x in 0..finite.width() as i32 {
                        if let Some(tile) = finite.get_tile(x, y) {
                            spawn_tile(&mut commands, &tilesets, tile, x, y, origin, tile_size, z);
                        }
                    }
                }
            }
            TileLayer::Infinite(infinite) => {
                let (chunk_w, chunk_h) = (ChunkData::WIDTH as i32, ChunkData::HEIGHT as i32);
                for (chunk_pos, chunk) in infinite.chunks() {
                    for cy in 0..chunk_h {
                        for cx in 0..chunk_w {
                            if let Some(tile) = chunk.get_tile(cx, cy) {
                                let x = chunk_pos.0 * chunk_w + cx;
                                let y = chunk_pos.1 * chunk_h + cy;
                                spawn_tile(
                                    &mut commands, &tilesets, tile, x, y, origin, tile_size, z,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Spawns one tile sprite at tile coordinates `(x, y)`.
///
/// `origin` is the world position of tile `(0, 0)` and `tile_size` the grid cell
/// size; tiles are laid out right and down from the origin.
fn spawn_tile(
    commands: &mut Commands,
    tilesets: &[Option<TilesetAssets>],
    tile: LayerTile,
    x: i32,
    y: i32,
    origin: Vec2,
    tile_size: Vec2,
    z: f32,
) {
    let Some(assets) = &tilesets[tile.tileset_index()] else {
        return;
    };

    commands.spawn((
        Sprite {
            image: assets.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.layout.clone(),
                index: tile.id() as usize,
            }),
            flip_x: tile.flip_h,
            flip_y: tile.flip_v,
            ..default()
        },
        Transform::from_xyz(
            origin.x + x as f32 * tile_size.x,
            origin.y - y as f32 * tile_size.y,
            z,
        ),
    ));
}

/// Builds image and atlas handles for every single-image tileset of `map`.
///
/// Image-collection tilesets (one image per tile) map to `None` and are skipped.
fn build_tileset_assets(
    map: &tiled::Map,
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    assets_dir: &Path,
) -> Vec<Option<TilesetAssets>> {
    map.tilesets()
        .iter()
        .map(|tileset| {
            let image = tileset.image.as_ref()?;

            let columns = tileset.columns.max(1);
            let rows = tileset.tilecount.div_ceil(columns);
            let layout = layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(tileset.tile_width, tileset.tile_height),
                columns,
                rows,
                Some(UVec2::splat(tileset.spacing)),
                Some(UVec2::splat(tileset.margin)),
            ));

            Some(TilesetAssets {
                image: asset_server.load(asset_path(assets_dir, &image.source)),
                layout,
            })
        })
        .collect()
}

/// Converts a Tiled image path (resolved against the map file) into a path the
/// [`AssetServer`] understands, i.e. relative to the `assets/` root.
fn asset_path(assets_dir: &Path, source: &Path) -> PathBuf {
    source
        .strip_prefix(assets_dir)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| source.to_path_buf())
}
