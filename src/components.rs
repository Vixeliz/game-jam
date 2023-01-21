use bevy::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum GameState {
    Splashscreen,
    Menu,
    Game,
}

// Tags for all the different states
#[derive(Component)]
pub struct Splashscreen;
#[derive(Component)]
pub struct Menu;
#[derive(Component)]
pub struct Game;
