//! A 2D camera that smoothly follows the player.

use bevy::prelude::*;

use crate::player::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, follow_player);
    }
}

fn spawn_camera(mut commands: Commands) {
    // Keep the camera on the origin in z as well: `Camera2d` sees the window
    // `(-1000, 1000]`, so the player (z = 1) and the map layers (z < 0) all fit.
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
}

fn follow_player(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let target = player_transform.translation.truncate();
    let current = camera_transform.translation.truncate();
    // Exponential-ish smoothing, clamped so it never overshoots.
    let smoothed = current.lerp(target, (10.0 * time.delta_secs()).min(1.0));

    camera_transform.translation.x = smoothed.x;
    camera_transform.translation.y = smoothed.y;
}
