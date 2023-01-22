use std::collections::{HashMap, HashSet};

use bevy::ecs::world;
use bevy::render::camera::{self, RenderTarget, Viewport};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::RenderLayers;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::{asset, prelude::*};
use bevy_ecs_ldtk::prelude::LayerInstance;
use bevy_ecs_ldtk::{GridCoords, LdtkLevel, LdtkWorldBundle, LevelSelection};
use bevy_rapier2d::prelude::*;
use iyes_loopless::state::NextState;
use sark_pathfinding::*;

use crate::components::{
    ColliderBundle, Game, GameState, InGame, Player, PostProcessingMaterial, Wall,
};
use crate::states::game::components::*;

pub fn input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut ingame: ResMut<InGame>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::Menu));
        ingame.0 = false;
    }
}

pub fn aiming(
    mut windows: ResMut<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CameraTag>>,
    mut world_coords: ResMut<WorldMouseCoords>,
) {
    // Games typically only have one window (the primary window).
    // For multi-window applications, you need to use a specific window ID here.
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let windows = if let RenderTarget::Window(id) = camera.target {
        windows.get_mut(id).unwrap()
    } else {
        windows.get_primary_mut().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = windows.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(windows.width() as f32, windows.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();
        world_coords.0 = world_pos;
    }
}

pub fn unhide_cursor(mut windows: ResMut<Windows>) {
    let windows = windows.get_primary_mut().unwrap();
    windows.set_cursor_visibility(true);
}

pub fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    for (level_handle, level_transform) in &level_query {
        if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
            let level_bounds = Rect {
                min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
                max: Vec2::new(
                    level_transform.translation.x + ldtk_level.level.px_wid as f32,
                    level_transform.translation.y + ldtk_level.level.px_hei as f32,
                ),
            };
            for player_transform in &player_query {
                if player_transform.translation.x < level_bounds.max.x
                    && player_transform.translation.x > level_bounds.min.x
                    && player_transform.translation.y < level_bounds.max.y
                    && player_transform.translation.y > level_bounds.min.y
                    && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
}

pub fn scale_render_image(
    mut texture_query: Query<&mut Transform, With<RenderImage>>,
    mut camera_query: Query<&mut Camera, (Without<Player>, Without<CameraTag>)>,
    mut windows: ResMut<Windows>,
) {
    if let Ok(mut camera) = camera_query.get_single_mut() {
        let window = windows.primary_mut();
        let mut texture_transform = texture_query.single_mut();
        let window_size: UVec2 = if window.physical_height() > window.physical_width()
            || window.physical_height() as f32 * ASPECT_RATIO > window.physical_width() as f32
        {
            UVec2 {
                x: window.physical_width(),
                y: (window.physical_width() as f32 / ASPECT_RATIO).round() as u32,
            }
        } else {
            UVec2 {
                x: (window.physical_height() as f32 * ASPECT_RATIO).round() as u32,
                y: window.physical_height(),
            }
        };
        let scale_width = window_size.x / SCREEN_WIDTH;
        let scale_height = window_size.y / SCREEN_HEIGHT;
        let window_position: UVec2 = if window.physical_height() > window.physical_width()
            || window.physical_height() as f32 * ASPECT_RATIO > window.physical_width() as f32
        {
            UVec2 {
                x: 0,
                y: window.physical_height() / 2 - window_size.y / 2,
            }
        } else {
            UVec2 {
                x: window.physical_width() / 2 - window_size.x / 2,
                y: 0,
            }
        };
        texture_transform.scale = Vec3 {
            x: scale_width as f32,
            y: scale_height as f32,
            z: 1.0,
        };
        camera.viewport = Some(Viewport {
            physical_size: window_size,
            physical_position: window_position,
            ..Default::default()
        });
    }
}

pub fn move_player(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity), With<Player>>,
    colliding_query: Query<&CollidingEntities, With<Player>>,
    player_query: Query<(Entity, &Items, &Transform), With<Player>>,
    items_query: Query<(Entity, &Items), (With<ItemTag>, With<Collider>)>,
    asset_server: Res<AssetServer>,
) {
    for (mut velocity) in &mut query {
        let right = if input.pressed(KeyCode::D) { 1. } else { 0. };
        let left = if input.pressed(KeyCode::A) { 1. } else { 0. };

        velocity.linvel.x = (right - left) * 200.;
        let up = if input.pressed(KeyCode::W) { 1. } else { 0. };
        let down = if input.pressed(KeyCode::S) { 1. } else { 0. };

        velocity.linvel.y = (up - down) * 200.;
    }
    if input.just_pressed(KeyCode::E) {
        for collider_entities in colliding_query.iter() {
            for collider in collider_entities.iter() {
                if let Ok((item_entity, item_type)) = items_query.get(collider) {
                    if let Ok((player, player_item, player_transform)) = player_query.get_single() {
                        switch_item(
                            item_type,
                            &mut commands,
                            player,
                            player_item,
                            item_entity,
                            player_transform,
                            &asset_server,
                        );
                        return;
                    }
                }
            }
        }
    }
}

fn switch_item(
    item_type: &Items,
    mut commands: &mut Commands,
    player: Entity,
    player_item: &Items,
    item_entity: Entity,
    player_transform: &Transform,
    asset_server: &Res<AssetServer>,
) {
    match item_type {
        Items::None => println!("Not implemented"),
        _ => {
            match player_item {
                Items::GlassBottle => {
                    commands
                        .spawn((GlassBottle {
                            sprite_bundle: SpriteBundle {
                                transform: Transform {
                                    translation: player_transform.translation,
                                    rotation: Quat {
                                        x: player_transform.rotation.x,
                                        y: player_transform.rotation.y,
                                        z: 0.0,
                                        w: player_transform.rotation.w,
                                    },
                                    ..Default::default()
                                },
                                texture: asset_server.load("glass_bottle.png"),
                                ..Default::default()
                            },
                            ..Default::default()
                        },))
                        .insert(Items::GlassBottle);
                }
                Items::None => {
                    println!("No Item!");
                }
                _ => println!("Todo"),
            }
            commands.get_entity(player).unwrap().insert(*item_type);
            commands.get_entity(item_entity).unwrap().despawn();
        }
    }
}

pub fn show_held_item(
    mut commands: Commands,
    mut commands2: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(Entity, &Items), With<Player>>,
    previous_child_query: Query<(Entity), (With<ItemTag>, Without<Collider>)>,
    last_item: Res<Items>,
) {
    if let Ok((player, player_item)) = player_query.get_single() {
        if last_item.into_inner() != player_item {
            if let Ok(previous_child) = previous_child_query.get_single() {
                commands.get_entity(previous_child).unwrap().despawn();
            }
            commands.insert_resource(*player_item);
            let item_path = match player_item {
                Items::GlassBottle => "glass_bottle.png",
                Items::None => {
                    return;
                }
                _ => "missing.png",
            };
            //TODO: Replace hardcoded path with a match that will set a string to correct image
            let child_sprite = commands.spawn((
                SpriteBundle {
                    transform: Transform::from_translation(Vec3 {
                        x: 8.0,
                        y: 0.0,
                        z: 4.0,
                    }),
                    texture: asset_server.load(item_path),
                    ..Default::default()
                },
                ItemTag,
                Player,
            ));
            commands2
                .get_entity(player)
                .unwrap()
                .add_child(child_sprite.id());
            println!("Showing New Item");
        }
    }
}

pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        (Without<Player>, With<CameraTag>),
    >,
    player_query: Query<&Transform, (With<Player>, Without<ItemTag>)>,
    level_query: Query<
        (&Transform, &Handle<LdtkLevel>),
        (Without<OrthographicProjection>, Without<Player>),
    >,
    level_selection: Res<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();
        for (level_transform, level_handle) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level = &ldtk_level.level;
                if level_selection.is_match(&0, level) {
                    orthographic_projection.scaling_mode = bevy::render::camera::ScalingMode::None;
                    orthographic_projection.bottom = 0.;
                    orthographic_projection.left = 0.;

                    orthographic_projection.right = SCREEN_HEIGHT as f32 * ASPECT_RATIO;
                    camera_transform.translation.x = (player_translation.x
                        - level_transform.translation.x
                        - orthographic_projection.right / 2.)
                        .clamp(0., level.px_wid as f32 - orthographic_projection.right);
                    orthographic_projection.top = SCREEN_WIDTH as f32 / ASPECT_RATIO;
                    camera_transform.translation.y = (player_translation.y
                        - level_transform.translation.y
                        - orthographic_projection.top / 2.)
                        .clamp(0., level.px_hei as f32 - orthographic_projection.top);

                    camera_transform.translation.x += level_transform.translation.x;
                    camera_transform.translation.y += level_transform.translation.y;
                }
            }
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut ingame: ResMut<InGame>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if ingame.0 {
    } else {
        ingame.0 = true;
        let size = Extent3d {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            ..default()
        };

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);

        let image_handle = images.add(image);

        commands.spawn((
            Game,
            LdtkWorldBundle {
                ldtk_handle: asset_server.load("Map.ldtk"),
                ..Default::default()
            },
        ));
        commands.spawn((
            Game,
            CameraTag,
            UiCameraConfig { show_ui: false },
            Camera2dBundle {
                camera: Camera {
                    target: RenderTarget::Image(image_handle.clone()),
                    ..default()
                },
                ..Default::default()
            },
        ));
        let post_processing_pass_layer =
            RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            size.width as f32,
            size.height as f32,
        ))));

        // This material has the texture that has been rendered.
        let material_handle = post_processing_materials.add(PostProcessingMaterial {
            source_image: image_handle,
        });

        // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: quad_handle.into(),
                material: material_handle,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.5),
                    ..default()
                },
                ..default()
            },
            post_processing_pass_layer,
            RenderImage,
            Game,
        ));

        commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    viewport: Some(Viewport {
                        physical_size: UVec2 {
                            x: SCREEN_WIDTH,
                            y: SCREEN_HEIGHT,
                        },
                        ..Default::default()
                    }),
                    // renders after the first main camera which has default value: 0.
                    priority: 1,
                    ..default()
                },
                ..Camera2dBundle::default()
            },
            post_processing_pass_layer,
            Game,
        ));
    }
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

pub fn fix_player_col(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    collider_query: Query<(&Collider), With<Player>>,
) {
    if let Ok(collider) = collider_query.get_single() {
    } else {
        if let Ok(player) = player_query.get_single() {
            let rotation_constraints = LockedAxes::ROTATION_LOCKED;
            commands.get_entity(player).unwrap().insert(ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints,
                ..Default::default()
            });
        }
    }
}

pub fn fix_enemy_col(
    mut commands: Commands,
    main_enemy_query: Query<Entity, With<MainEnemy>>,
    collider_query: Query<(&Collider), With<MainEnemy>>,
) {
    if let Ok(collider) = collider_query.get_single() {
    } else {
        if let Ok(enemy) = main_enemy_query.get_single() {
            let rotation_constraints = LockedAxes::ROTATION_LOCKED;
            commands.get_entity(enemy).unwrap().insert(ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints,
                ..Default::default()
            });
        }
    }
}

pub fn add_item_col(
    mut commands: Commands,
    item_query: Query<Entity, (With<ItemTag>, Without<Player>)>,
    collider_query: Query<(&Collider), With<ItemTag>>,
) {
    for item in item_query.iter() {
        if let Ok(col) = collider_query.get(item) {
        } else {
            let rotation_constraints = LockedAxes::ROTATION_LOCKED;
            commands.get_entity(item).unwrap().insert(SensorBundle {
                collider: Collider::cuboid(8., 8.),
                sensor: Sensor,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
                ..Default::default()
            });
        }
    }
}

pub fn face_towards_cursor(
    mut player_query: Query<&mut Transform, (With<Player>, Without<ItemTag>)>,
    world_coords: Res<WorldMouseCoords>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let diff = Vec3 {
            x: world_coords.0.x,
            y: world_coords.0.y,
            z: 0.0,
        } - player_transform.translation;
        let angle = diff.y.atan2(diff.x) - 1.570796;
        player_transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), angle);
    }
}

pub fn cursor(mut commands: Commands, world_coords: Res<WorldMouseCoords>) {}

pub fn create_collision_map(
    mut commands: Commands,
    wall_query: Query<(&GridCoords), Added<Wall>>,
    mut path_map: ResMut<PathfindingMap>,
    mut astar_map: ResMut<AstarMap>,
    mut path_map_initialized: ResMut<PathInit>,
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    if path_map_initialized.0 != true {
        for (level_handle, level_transform) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                if let Some(level) = ldtk_levels.get(level_handle) {
                    let grid_size_y = level.level.px_hei / 16;
                    let grid_size_x = level.level.px_wid / 16;
                    let mut map = PathMap2d::new([
                        grid_size_x.try_into().unwrap(),
                        grid_size_y.try_into().unwrap(),
                    ]);
                    let astarm: AStar<[i32; 2]> = AStar::from_size([
                        grid_size_x.try_into().unwrap(),
                        grid_size_y.try_into().unwrap(),
                    ]);
                    for wall in wall_query.iter() {
                        map.set_obstacle([wall.x, wall.y], true);
                        path_map_initialized.0 = true;
                    }
                    commands.insert_resource(PathfindingMap { path_map: map });
                    commands.insert_resource(AstarMap { astar: astarm });
                }
            }
        }
    }
}

pub fn main_enemy_move(
    mut path_map: ResMut<PathfindingMap>,
    mut astar_map: ResMut<AstarMap>,
    mut main_enemy_query: Query<(&mut Transform, &mut Target, &mut TargetPath), With<MainEnemy>>,
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<MainEnemy>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    if let Ok((mut main_enemy_transform, mut enemy_target, mut enemy_path)) =
        main_enemy_query.get_single_mut()
    {
        let mut enemy_location = [0, 0];
        for (level_handle, level_transform) in &level_query {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                if level_selection.is_match(&0, &ldtk_level.level) {
                    let level_location = level_transform.translation;
                    let level_size = Vec2 {
                        x: ldtk_level.level.px_hei as f32,
                        y: ldtk_level.level.px_wid as f32,
                    };
                    enemy_location = convert_world_to_grid(
                        &level_location,
                        &level_size,
                        &Vec2 {
                            x: main_enemy_transform.translation.x,
                            y: main_enemy_transform.translation.y,
                        },
                    );
                }
            }
        }
        let backup_vector_path: Vec<[i32; 2]> = vec![[0, 0]];
        enemy_path.0 = astar_map
            .astar
            .find_path(&path_map.path_map, enemy_location, [8, 18])
            .unwrap_or_else(|| &&backup_vector_path)
            .to_vec();
        astar_map.astar.clear();
        astar_map.astar = AStar::from_size([
            path_map.path_map.width().try_into().unwrap(),
            path_map.path_map.height().try_into().unwrap(),
        ]);
    }
}

fn convert_world_to_grid(level_location: &Vec3, level_size: &Vec2, target: &Vec2) -> [i32; 2] {
    if target.x > level_location.x
        && target.x < level_location.x + level_size.x
        && target.y > level_location.y
        && target.y < level_location.y + level_size.y
    {
        return [
            ((target.x - level_location.x).abs() / 16.) as i32,
            ((target.y - level_location.y).abs() / 16.) as i32,
        ];
    } else {
        return [0, 0];
    }
}
