use bevy::prelude::*;
use bevy_ecs_ldtk::{EntityInstance, IntGridCell, LdtkEntity, LdtkIntCell};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, LockedAxes, Sensor};

use crate::components::Game;

#[derive(Component)]
pub struct CameraTag;

#[derive(Component)]
pub struct RenderImage;

pub const SCREEN_WIDTH: u32 = 448;
pub const SCREEN_HEIGHT: u32 = 256;
pub const ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;

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
