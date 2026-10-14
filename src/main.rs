mod animation;
mod assets;
mod camera;
mod player;
mod tilemap;

use bevy::prelude::*;

use animation::AnimationPlugin;
use camera::CameraPlugin;
use player::PlayerPlugin;
use tilemap::TilemapPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // Keep the pixel art crisp instead of blurry when scaled.
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tiny Swords".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.30, 0.55, 0.35)))
        .add_plugins((PlayerPlugin, AnimationPlugin, CameraPlugin, TilemapPlugin))
        .run();
}
