use bevy::{prelude::*, sprite::Material2dPlugin};
use bevy_ecs_ldtk::{
    prelude::RegisterLdtkObjects, LdtkPlugin, LdtkSettings, LevelBackground, LevelSelection,
    LevelSpawnBehavior, SetClearColor,
};
use bevy_rapier2d::{
    prelude::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_tweening::TweeningPlugin;
use iyes_loopless::prelude::*;
mod components;
use components::*;
mod systems;
use systems::*;
mod states;
use states::{
    game::components::{GlassBottle, Items},
    *,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(TweeningPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..Default::default()
        })
        .add_plugin(Material2dPlugin::<PostProcessingMaterial>::default())
        .add_plugin(LdtkPlugin)
        .insert_resource(LevelSelection::Index(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            level_background: LevelBackground::Nonexistent,
            ..Default::default()
        })
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
        .add_system(game::systems::fix_player_col.run_in_state(GameState::Game))
        .add_system(game::systems::add_item_col.run_in_state(GameState::Game))
        .add_system(game::systems::spawn_wall_collision.run_in_state(GameState::Game))
        .add_system(game::systems::scale_render_image.run_in_state(GameState::Game))
        .add_system(game::systems::move_player.run_in_state(GameState::Game))
        .add_system(game::systems::show_held_item.run_in_state(GameState::Game))
        .add_system(game::systems::update_level_selection.run_in_state(GameState::Game))
        .add_system(game::systems::camera_fit_inside_current_level.run_in_state(GameState::Game))
        .add_startup_system(systems::start)
        .add_system(print_current_state)
        .register_ldtk_entity::<PlayerBundle>("Player")
        .register_ldtk_entity::<GlassBottle>("GlassBottle")
        .register_ldtk_int_cell::<WallBundle>(1)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(InGame(false))
        .insert_resource(Items::GlassBottle)
        .run()
}
