use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{
    lens::{TransformRotateZLens, TransformScaleLens},
    Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween,
};
use iyes_loopless::prelude::*;

use crate::components::{GameState, Splashscreen};

#[derive(Resource)]
pub struct SplashTimer {
    timer: Timer,
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tween = Tween::new(
        EaseFunction::SineIn,
        Duration::from_secs(3),
        TransformScaleLens {
            start: Vec3::splat(0.9),
            end: Vec3::splat(1.0),
        },
    )
    .with_repeat_count(RepeatCount::Finite(1))
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);
    let tween_rotate = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        TransformRotateZLens {
            start: 0.0,
            end: 0.1,
        },
    )
    .with_repeat_count(RepeatCount::Finite(1))
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);
    let tween_rotate2 = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        TransformRotateZLens {
            start: 0.1,
            end: -0.1,
        },
    )
    .with_repeat_count(RepeatCount::Finite(1))
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);
    let sequence = tween.then(tween_rotate).then(tween_rotate2);
    commands.spawn((Splashscreen, Camera2dBundle::default()));
    commands.spawn((
        Splashscreen,
        SpriteBundle {
            texture: asset_server.load("Title.png"),
            transform: Transform::from_scale(Vec3::splat(0.0)),
            ..default()
        },
        Animator::new(sequence),
    ));
    commands.insert_resource(SplashTimer {
        // create the repeating timer
        timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
    });
}

pub fn update(mut commands: Commands, time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    timer.timer.tick(time.delta());
    if timer.timer.finished() {
        commands.insert_resource(NextState(GameState::Menu));
    }
}
pub fn input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::Menu));
    }
}
