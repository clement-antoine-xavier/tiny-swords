//! The player-controlled Warrior: spawning, input, movement, actions, animation.

use bevy::prelude::*;

use crate::animation::{AnimationConfig, AnimationLibrary};

/// Movement speed in pixels per second.
const SPEED: f32 = 160.0;

/// Clip names as authored in the Tiled `Animations` layer (`assets/Main.tmx`).
const IDLE: &str = "Idle";
const RUN: &str = "Run";
const GUARD: &str = "Guard";
const ATTACK1: &str = "Attack1";
const ATTACK2: &str = "Attack2";

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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            // The animation library is parsed from Tiled before the player uses it.
            spawn_player.after(crate::animation::build_animation_library),
        )
        .add_systems(
            Update,
            (read_move_input, resolve_action, apply_movement, update_animation).chain(),
        );
    }
}

/// Spawns the Warrior, playing the `Idle` clip defined in Tiled.
fn spawn_player(mut commands: Commands, library: Res<AnimationLibrary>) {
    let Some(clip) = library.get(IDLE) else {
        error!("animation `{IDLE}` is not defined in Tiled's `Animations` layer");
        return;
    };

    let mut sprite = Sprite {
        image: clip.image.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: clip.layout.clone(),
            index: clip.first,
        }),
        ..default()
    };
    let mut animation = AnimationConfig::default();
    animation.play(IDLE, clip, &mut sprite);

    commands.spawn((
        sprite,
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

/// Maps the current action (and movement) onto a Tiled animation clip.
fn update_animation(
    library: Res<AnimationLibrary>,
    mut query: Query<(&Velocity, &Action, &mut AnimationConfig, &mut Sprite), With<Player>>,
) {
    for (velocity, action, mut config, mut sprite) in &mut query {
        let name = match action.state {
            ActionState::Locomotion if velocity.0 == Vec2::ZERO => IDLE,
            ActionState::Locomotion => RUN,
            ActionState::Guard => GUARD,
            ActionState::Attack { step: 0 } => ATTACK1,
            ActionState::Attack { .. } => ATTACK2,
        };

        let Some(clip) = library.get(name) else {
            warn_once!("animation `{name}` is not defined in Tiled's `Animations` layer");
            continue;
        };

        config.play(name, clip, &mut sprite);
    }
}
