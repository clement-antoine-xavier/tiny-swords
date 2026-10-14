//! The player-controlled Warrior: spawning, input, movement, actions, animation.

use bevy::prelude::*;

use crate::animation::{AnimId, AnimationConfig};

/// Movement speed in pixels per second.
const SPEED: f32 = 160.0;

/// Every Warrior sprite sheet uses 192x192 frames.
const FRAME_SIZE: UVec2 = UVec2::splat(192);

const IDLE_FRAMES: u32 = 8;
const RUN_FRAMES: u32 = 6;
const GUARD_FRAMES: u32 = 6;
const ATTACK1_FRAMES: u32 = 4;
const ATTACK2_FRAMES: u32 = 4;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

/// The high-level action the Warrior is committed to.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ActionState {
    #[default]
    Locomotion,
    Guard,
    Attack { step: u8 },
}

/// The action the Warrior is currently committed to.
#[derive(Component, Default)]
pub struct Action {
    state: ActionState,
}

/// Texture and atlas handles shared by every player animation.
#[derive(Resource)]
struct PlayerAssets {
    idle_image: Handle<Image>,
    run_image: Handle<Image>,
    guard_image: Handle<Image>,
    attack1_image: Handle<Image>,
    attack2_image: Handle<Image>,
    idle_layout: Handle<TextureAtlasLayout>,
    run_layout: Handle<TextureAtlasLayout>,
    guard_layout: Handle<TextureAtlasLayout>,
    attack1_layout: Handle<TextureAtlasLayout>,
    attack2_layout: Handle<TextureAtlasLayout>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_assets, spawn_player).chain())
            .add_systems(
                Update,
                (read_move_input, resolve_action, apply_movement, update_animation).chain(),
            );
    }
}

fn setup_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let row = |columns: u32| TextureAtlasLayout::from_grid(FRAME_SIZE, columns, 1, None, None);

    let idle_layout = atlas_layouts.add(row(IDLE_FRAMES));
    let run_layout = atlas_layouts.add(row(RUN_FRAMES));
    let guard_layout = atlas_layouts.add(row(GUARD_FRAMES));
    let attack1_layout = atlas_layouts.add(row(ATTACK1_FRAMES));
    let attack2_layout = atlas_layouts.add(row(ATTACK2_FRAMES));

    commands.insert_resource(PlayerAssets {
        idle_image: asset_server.load("Units/Black Units/Warrior/Warrior_Idle.png"),
        run_image: asset_server.load("Units/Black Units/Warrior/Warrior_Run.png"),
        guard_image: asset_server.load("Units/Black Units/Warrior/Warrior_Guard.png"),
        attack1_image: asset_server.load("Units/Black Units/Warrior/Warrior_Attack1.png"),
        attack2_image: asset_server.load("Units/Black Units/Warrior/Warrior_Attack2.png"),
        idle_layout,
        run_layout,
        guard_layout,
        attack1_layout,
        attack2_layout,
    });
}

fn spawn_player(mut commands: Commands, assets: Res<PlayerAssets>) {
    let mut animation = AnimationConfig::default();
    animation.play(AnimId::Idle);

    commands.spawn((
        Sprite {
            image: assets.idle_image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.idle_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player,
        Velocity::default(),
        Action::default(),
        animation,
    ));
}

/// Turns held keys into a normalized direction (diagonals don't move faster).
fn read_move_input(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    let mut direction = Vec2::ZERO;

    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        direction.y += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        direction.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        direction.x -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        direction.x += 1.0;
    }

    let velocity = direction.normalize_or_zero() * SPEED;
    for mut current in &mut query {
        current.0 = velocity;
    }
}

/// Resolves input and the current animation's completion into an [`ActionState`].
fn resolve_action(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Action, &mut Velocity, &AnimationConfig), With<Player>>,
) {
    let attack_pressed =
        keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);
    let guard_held = keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight)
        || mouse.pressed(MouseButton::Right);

    for (mut action, mut velocity, animation) in &mut query {
        match action.state {
            // An attack plays to completion; extra presses are ignored.
            ActionState::Attack { .. } if !animation.is_finished() => {}
            // Attack1 finished: chain straight into Attack2.
            ActionState::Attack { step: 0 } => {
                action.state = ActionState::Attack { step: 1 };
            }
            // Attack2 finished: guard if held, else back to locomotion.
            ActionState::Attack { .. } => {
                action.state = if guard_held {
                    ActionState::Guard
                } else {
                    ActionState::Locomotion
                };
            }
            // Free to start a new action.
            _ => {
                if attack_pressed {
                    action.state = ActionState::Attack { step: 0 };
                } else if guard_held {
                    action.state = ActionState::Guard;
                } else {
                    action.state = ActionState::Locomotion;
                }
            }
        }

        // Guarding and attacking root the Warrior in place.
        if action.state != ActionState::Locomotion {
            velocity.0 = Vec2::ZERO;
        }
    }
}

/// Moves the player and mirrors the sprite to face the direction of travel.
fn apply_movement(
    time: Res<Time>,
    mut query: Query<(&Velocity, &Action, &mut Transform, &mut Sprite), With<Player>>,
) {
    for (velocity, action, mut transform, mut sprite) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        // Facing is locked once committed to an attack.
        if matches!(action.state, ActionState::Attack { .. }) {
            continue;
        }

        if velocity.0.x > 0.0 {
            sprite.flip_x = false;
        } else if velocity.0.x < 0.0 {
            sprite.flip_x = true;
        }
    }
}

/// Maps the current action (and movement) onto an animation, swapping sheets on change.
fn update_animation(
    assets: Res<PlayerAssets>,
    mut query: Query<(&Velocity, &Action, &mut AnimationConfig, &mut Sprite), With<Player>>,
) {
    for (velocity, action, mut config, mut sprite) in &mut query {
        let anim = match action.state {
            ActionState::Locomotion if velocity.0 == Vec2::ZERO => AnimId::Idle,
            ActionState::Locomotion => AnimId::Run,
            ActionState::Guard => AnimId::Guard,
            ActionState::Attack { step: 0 } => AnimId::Attack1,
            ActionState::Attack { .. } => AnimId::Attack2,
        };

        if !config.play(anim) {
            continue;
        }

        let (image, layout) = match anim {
            AnimId::Idle => (assets.idle_image.clone(), assets.idle_layout.clone()),
            AnimId::Run => (assets.run_image.clone(), assets.run_layout.clone()),
            AnimId::Guard => (assets.guard_image.clone(), assets.guard_layout.clone()),
            AnimId::Attack1 => (assets.attack1_image.clone(), assets.attack1_layout.clone()),
            AnimId::Attack2 => (assets.attack2_image.clone(), assets.attack2_layout.clone()),
        };

        sprite.image = image;
        sprite.texture_atlas = Some(TextureAtlas { layout, index: 0 });
    }
}
