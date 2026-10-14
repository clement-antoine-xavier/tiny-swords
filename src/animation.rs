//! A tiny, dependency-free sprite-sheet animator.
//!
//! Each animated entity owns an [`AnimationConfig`] describing which frames of
//! its atlas to loop through and at what speed. Swapping between animations is
//! done by calling [`AnimationConfig::play`]; the caller is responsible for
//! pointing the [`Sprite`] at the matching image/atlas layout.
//!
//! Animations are either looping (Idle/Run/Guard) or one-shot (Attacks), which
//! hold their final frame and report completion via [`AnimationConfig::is_finished`].

use std::time::Duration;

use bevy::prelude::*;

/// The animations the Warrior can play.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnimId {
    Idle,
    Run,
    Guard,
    Attack1,
    Attack2,
}

/// Frame range and playback settings for a single animation.
struct AnimSpec {
    first: usize,
    last: usize,
    fps: u8,
    looping: bool,
}

impl AnimId {
    fn spec(self) -> AnimSpec {
        match self {
            AnimId::Idle => AnimSpec { first: 0, last: 7, fps: 8, looping: true },
            AnimId::Run => AnimSpec { first: 0, last: 5, fps: 12, looping: true },
            AnimId::Guard => AnimSpec { first: 0, last: 5, fps: 8, looping: true },
            AnimId::Attack1 => AnimSpec { first: 0, last: 3, fps: 12, looping: false },
            AnimId::Attack2 => AnimSpec { first: 0, last: 3, fps: 12, looping: false },
        }
    }
}

/// Drives a sprite-sheet animation on a [`Sprite`].
#[derive(Component)]
pub struct AnimationConfig {
    current: Option<AnimId>,
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
            fps: 8,
            looping: true,
            finished: false,
            frame_timer: Self::timer_from_fps(8),
        }
    }
}

impl AnimationConfig {
    /// Switch to `anim`. Returns `true` if the active animation changed, so the
    /// caller knows it must update the sprite's image and atlas layout.
    pub fn play(&mut self, anim: AnimId) -> bool {
        if self.current == Some(anim) {
            return false;
        }

        let spec = anim.spec();
        self.current = Some(anim);
        self.first = spec.first;
        self.last = spec.last;
        self.fps = spec.fps;
        self.looping = spec.looping;
        self.finished = false;
        self.frame_timer = Self::timer_from_fps(spec.fps);
        true
    }

    /// Whether a one-shot animation has reached (and is holding) its last frame.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / fps as f32), TimerMode::Repeating)
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, advance_frames);
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
            // A one-shot animation is done once it lands on its final frame.
            if !config.looping && atlas.index == config.last {
                config.finished = true;
            }
        } else if config.looping {
            atlas.index = config.first;
        }
    }
}
