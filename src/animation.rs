//! Data-driven sprite-sheet animation.
//!
//! Animation clips are authored in Tiled, not in code: the `Animations` object
//! layer of `assets/Main.tmx` holds one object per clip, and its custom
//! properties describe the sprite sheet, the frame range, the playback speed
//! and whether the clip loops. [`build_animation_library`] parses that layer
//! into an [`AnimationLibrary`] at startup, and gameplay simply plays clips by
//! name.
//!
//! Swapping clips is done with [`AnimationConfig::play`], which also points the
//! [`Sprite`] at the clip's sheet and first frame. Clips are either looping
//! (Idle/Run/Guard) or one-shot (the attacks), which hold their final frame and
//! report completion via [`AnimationConfig::is_finished`].

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use bevy::prelude::*;
use tiled::{Object, PropertyValue};

use crate::assets::ANIM_FPS;
use crate::tilemap::{LevelMap, asset_path, assets_dir, load_map};

/// Name of the Tiled object layer that defines the animation clips.
const ANIMATION_LAYER: &str = "Animations";

/// One animation clip: a sprite sheet plus the frames to play from it.
#[derive(Clone)]
pub struct AnimationClip {
    /// The sprite sheet image.
    pub image: Handle<Image>,
    /// Atlas layout of the sheet, one row of equally sized frames.
    pub layout: Handle<TextureAtlasLayout>,
    /// First frame index of the clip, inclusive.
    pub first: usize,
    /// Last frame index of the clip, inclusive.
    pub last: usize,
    /// Playback rate, in frames per second.
    pub fps: u8,
    /// Whether the clip loops or holds its last frame (a one-shot).
    pub looping: bool,
}

/// Every clip authored in Tiled, keyed by the animation object's name.
#[derive(Resource, Default)]
pub struct AnimationLibrary(HashMap<String, AnimationClip>);

impl AnimationLibrary {
    /// Looks up the clip named `name`.
    pub fn get(&self, name: &str) -> Option<&AnimationClip> {
        self.0.get(name)
    }
}

/// Drives a sprite-sheet animation on a [`Sprite`].
#[derive(Component)]
pub struct AnimationConfig {
    current: Option<String>,
    first: usize,
    last: usize,
    fps: u8,
    looping: bool,
    finished: bool,
    frame_timer: Timer,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            current: None,
            first: 0,
            last: 0,
            fps: ANIM_FPS,
            looping: true,
            finished: false,
            frame_timer: Self::timer_from_fps(ANIM_FPS),
        }
    }
}

impl AnimationConfig {
    /// Switches to the clip named `name`, pointing `sprite` at its sheet.
    ///
    /// Returns `true` if the active clip changed, so the caller knows the sprite
    /// was updated.
    pub fn play(&mut self, name: &str, clip: &AnimationClip, sprite: &mut Sprite) -> bool {
        if self.current.as_deref() == Some(name) {
            return false;
        }

        self.current = Some(name.to_owned());
        self.first = clip.first;
        self.last = clip.last;
        self.fps = clip.fps;
        self.looping = clip.looping;
        self.finished = false;
        self.frame_timer = Self::timer_from_fps(clip.fps);

        sprite.image = clip.image.clone();
        sprite.texture_atlas = Some(TextureAtlas {
            layout: clip.layout.clone(),
            index: clip.first,
        });

        true
    }

    /// Whether a one-shot clip has reached (and is holding) its last frame.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(
            Duration::from_secs_f32(1.0 / fps.max(1) as f32),
            TimerMode::Repeating,
        )
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_animation_library.after(load_map))
            .add_systems(Update, advance_frames);
    }
}

/// Steps every animated sprite to its next frame when its timer elapses.
fn advance_frames(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        config.frame_timer.tick(time.delta());

        if !config.frame_timer.just_finished() {
            continue;
        }

        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        if atlas.index < config.last {
            atlas.index += 1;
            // A one-shot clip is done once it lands on its final frame.
            if !config.looping && atlas.index == config.last {
                config.finished = true;
            }
        } else if config.looping {
            atlas.index = config.first;
        }
    }
}

/// Parses the [`ANIMATION_LAYER`] object layer of [`LevelMap`] into an
/// [`AnimationLibrary`].
///
/// The library is always inserted — empty when the map failed to load or
/// defines no clips — so downstream systems can depend on the resource
/// unconditionally.
pub(crate) fn build_animation_library(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    level: Option<Res<LevelMap>>,
) {
    let mut library = AnimationLibrary::default();

    if let Some(level) = level {
        let assets_dir = assets_dir();

        for layer in level.0.layers() {
            if layer.name.as_str() != ANIMATION_LAYER {
                continue;
            }

            let Some(objects) = layer.as_object_layer() else {
                warn!("`{ANIMATION_LAYER}` is not an object layer; no clips loaded");
                continue;
            };

            for object in objects.objects() {
                if object.name.is_empty() {
                    warn!("skipping unnamed animation object");
                    continue;
                }

                match parse_clip(object, &asset_server, &mut layouts, &assets_dir) {
                    Ok(clip) => {
                        library.0.insert(object.name.clone(), clip);
                    }
                    Err(error) => warn!("skipping animation `{}`: {error}", object.name),
                }
            }
        }
    }

    info!("loaded {} animation clip(s) from Tiled", library.0.len());
    commands.insert_resource(library);
}

/// Builds an [`AnimationClip`] from one `Animations` object's properties.
fn parse_clip(
    object: Object,
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    assets_dir: &Path,
) -> Result<AnimationClip, String> {
    let image = string_prop(&object, "image").ok_or("missing `image` property")?;
    let frame_size = UVec2::new(
        int_prop(&object, "frame_width")?,
        int_prop(&object, "frame_height")?,
    );
    let columns = int_prop(&object, "columns")?.max(1);
    let first = int_prop(&object, "first")? as usize;
    let last = int_prop(&object, "last")? as usize;

    if last < first {
        return Err(format!("`last` ({last}) precedes `first` ({first})"));
    }

    let fps = u32_prop(&object, "fps")
        .unwrap_or(ANIM_FPS as u32)
        .clamp(1, u8::MAX as u32) as u8;
    let looping = bool_prop(&object, "looping").unwrap_or(true);

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        frame_size, columns, 1, None, None,
    ));

    Ok(AnimationClip {
        image: asset_server.load(asset_path(assets_dir, &assets_dir.join(image))),
        layout,
        first,
        last,
        fps,
        looping,
    })
}

/// Reads a required non-negative integer property.
fn int_prop(object: &Object, key: &str) -> Result<u32, String> {
    u32_prop(object, key).ok_or_else(|| format!("missing integer property `{key}`"))
}

/// Reads an optional non-negative integer property.
fn u32_prop(object: &Object, key: &str) -> Option<u32> {
    match object.properties.get(key)? {
        PropertyValue::IntValue(value) if *value >= 0 => Some(*value as u32),
        PropertyValue::StringValue(value) => value.parse().ok(),
        _ => None,
    }
}

/// Reads an optional boolean property.
fn bool_prop(object: &Object, key: &str) -> Option<bool> {
    match object.properties.get(key)? {
        PropertyValue::BoolValue(value) => Some(*value),
        PropertyValue::StringValue(value) => value.parse().ok(),
        _ => None,
    }
}

/// Reads an optional string or file property.
fn string_prop(object: &Object, key: &str) -> Option<String> {
    match object.properties.get(key)? {
        PropertyValue::StringValue(value) | PropertyValue::FileValue(value) => Some(value.clone()),
        _ => None,
    }
}
