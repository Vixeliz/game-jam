use bevy::prelude::*;
use bevy_ecs_ldtk::{EntityInstance, IntGridCell, LdtkEntity, LdtkIntCell, Worldly};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollidingEntities, LockedAxes, Sensor};
use sark_pathfinding::{AStar, PathMap2d};

use crate::components::Game;

#[derive(Component)]
pub struct CameraTag;

#[derive(Component)]
pub struct RenderImage;

pub const SCREEN_WIDTH: u32 = 448;
pub const SCREEN_HEIGHT: u32 = 256;
pub const ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;

#[derive(Component)]
pub struct CursorTag;

#[derive(Resource)]
pub struct WorldMouseCoords(pub Vec2);

#[derive(Clone, Component, Default, Debug, Copy, Resource, PartialEq, Eq)]
pub enum Items {
    #[default]
    None,
    GlassBottle,
    Shears,
}

#[derive(Component, Clone, Default)]
pub struct ItemTag;

#[derive(Bundle, Clone, Default)]
pub struct InteractableItem {
    pub tag: ItemTag,
    game: Game,
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct SensorBundle {
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
}

#[derive(Component)]
pub struct HeldItem(u32);

#[derive(Bundle, Default, Clone, LdtkEntity)]
pub struct GlassBottle {
    pub interactable: InteractableItem,
    #[sprite_bundle("glass_bottle.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    pub item: Items,
    #[from_entity_instance]
    pub entity_instance: EntityInstance,
}

impl From<EntityInstance> for Items {
    fn from(entity_instance: EntityInstance) -> Items {
        match entity_instance.identifier.as_ref() {
            "GlassBottle" => Items::GlassBottle,
            _ => Items::None,
        }
    }
}

#[derive(Component, Default)]
pub struct NoiseValue(f32);

#[derive(Resource)]
pub struct PathfindingMap {
    pub path_map: PathMap2d,
}
#[derive(Resource)]
pub struct AstarMap {
    pub astar: AStar<[i32; 2]>,
}
#[derive(Resource)]
pub struct PathInit(pub bool);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Enemy;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct MainEnemy;

#[derive(Default, Bundle, LdtkEntity)]
pub struct MainEnemyBundle {
    pub enemy_tag: MainEnemy,
    #[sprite_bundle("main_enemy.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    game: Game,
    #[worldly]
    pub worldly: Worldly,
    #[bundle]
    pub held_item: Items,
    pub colliding_entities: CollidingEntities,
    pub current_target: Target,
    pub target_path: TargetPath,
}

#[derive(Default, Component)]
pub struct Target(Vec2);

#[derive(Default, Component)]
pub struct TargetPath(pub Vec<[i32; 2]>);
