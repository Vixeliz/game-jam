use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;
use iyes_loopless::prelude::*;
mod components;
use components::*;
mod systems;
use systems::*;
mod states;
use states::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_loopless_state(GameState::Splashscreen)
        .add_enter_system(GameState::Splashscreen, splashscreen::setup)
        .add_enter_system(GameState::Menu, menu::setup)
        .add_enter_system(GameState::Game, game::systems::setup)
        .add_exit_system(GameState::Splashscreen, despawn_with::<Splashscreen>)
        .add_exit_system(GameState::Menu, despawn_with::<Menu>)
        .add_exit_system(GameState::Game, despawn_with::<Game>)
        .add_system(splashscreen::update.run_in_state(GameState::Splashscreen))
        .add_system(splashscreen::input.run_in_state(GameState::Splashscreen))
        .add_system(menu::input.run_in_state(GameState::Menu))
        .add_system(game::systems::input.run_in_state(GameState::Game))
        .add_startup_system(systems::start)
        .add_system(print_current_state)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .run()
}
