use bevy::prelude::*;
use iyes_loopless::state::CurrentState;

use crate::components::GameState;

pub fn start() {}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn print_current_state(state: Res<CurrentState<GameState>>) {
    if state.is_changed() {
        println!("{:?}", state.0);
    }
}
