use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};
use bevy_ecs_ldtk::{EntityInstance, LdtkEntity, LdtkIntCell, Worldly};
use bevy_rapier2d::prelude::*;

use crate::states::game::components::{Items, NoiseValue};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum GameState {
    Splashscreen,
    Menu,
    Game,
}
#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

// Tags for all the different states
#[derive(Component)]
pub struct Splashscreen;
#[derive(Component)]
pub struct Menu;
#[derive(Default, Component, Clone)]
pub struct Game;

#[derive(Resource)]
pub struct InGame(pub bool);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    pub player_tag: Player,
    #[sprite_bundle("player.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    game: Game,
    #[worldly]
    pub worldly: Worldly,
    #[bundle]
    pub held_item: Items,
    pub colliding_entities: CollidingEntities,
    pub noise: NoiseValue,
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "bc2f08eb-a0fb-43f1-a908-54871ea597d5"]
pub struct PostProcessingMaterial {
    /// In this example, this image will be the result of the main camera.
    #[texture(0)]
    #[sampler(1)]
    pub source_image: Handle<Image>,
}

impl Material2d for PostProcessingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material_chromatic_aberration.wgsl".into()
    }
}
