use bevy::prelude::*;
use bevy_ecs_ldtk::{IntGridCell, LdtkEntity, LdtkIntCell};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, LockedAxes, Sensor};

use crate::components::Game;

#[derive(Component)]
pub struct CameraTag;

#[derive(Component)]
pub struct RenderImage;

pub const SCREEN_WIDTH: u32 = 448;
pub const SCREEN_HEIGHT: u32 = 256;
pub const ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;

#[derive(Clone, Component, Default)]
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

#[derive(Bundle, Clone, LdtkEntity)]
pub struct GlassBottle {
    pub interactable: InteractableItem,
    #[sprite_bundle("glass_bottle.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    pub item: Items,
}

impl Default for GlassBottle {
    fn default() -> GlassBottle {
        let default_bottle: GlassBottle = GlassBottle {
            interactable: InteractableItem::default(),
            sprite_bundle: SpriteBundle::default(),
            item: Items::GlassBottle,
        };
        default_bottle
    }
}
