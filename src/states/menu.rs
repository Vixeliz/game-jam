use bevy::prelude::*;
use iyes_loopless::state::NextState;

use crate::components::{GameState, Menu};

pub fn setup(mut commands: Commands) {
    commands.spawn((Menu, Camera2dBundle::default()));
}

pub fn input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::Game));
    }
}
