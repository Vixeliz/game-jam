use bevy::prelude::Component;

#[derive(Component)]
pub struct CameraTag;

#[derive(Component)]
pub struct RenderImage;

pub const SCREEN_WIDTH: u32 = 448;
pub const SCREEN_HEIGHT: u32 = 256;
pub const ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
