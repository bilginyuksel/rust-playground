use bevy::{
    app::App,
    core::FixedTimestep,
    input::keyboard::KeyCode,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashMap;

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

#[derive(Component)]
struct Wall;

const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

impl WallBundle {
    fn new(loc: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: loc.position().extend(0.),
                    scale: loc.size().extend(1.),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

enum WallLocation {
    Bottom,
    Left,
    Right,
    Top,
}

const LEFT_WALL: f32 = -115.;
const RIGHT_WALL: f32 = 115.;
const BOTTOM_WALL: f32 = -250.;
const TOP_WALL: f32 = 250.;

const WALL_THICKNESS: f32 = 10.;

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let height = TOP_WALL - BOTTOM_WALL;
        let width = RIGHT_WALL - LEFT_WALL;

        match self {
            WallLocation::Left => Vec2::new(WALL_THICKNESS, height + WALL_THICKNESS),
            WallLocation::Right => Vec2::new(WALL_THICKNESS, height + WALL_THICKNESS),
            WallLocation::Top => Vec2::new(width + WALL_THICKNESS, WALL_THICKNESS),
            WallLocation::Bottom => Vec2::new(width + WALL_THICKNESS, WALL_THICKNESS),
        }
    }
}

#[derive(Component)]
struct Collider;

#[derive(Component, Deref, DerefMut)]
struct Gravity(Vec2);

impl Gravity {
    fn default() -> Gravity {
        Gravity(Vec2::new(0., 20.))
    }
}

#[derive(Component)]
struct Block;

#[derive(Component)]
struct GameObjects {
    objects: HashMap<Entity, Vec<Entity>>,
}

const FPS: f32 = 1.0;

// TODO:
//#[derive(Default)]
//struct WorldPlugin;
//
//impl Plugin for WorldPlugin {
//    fn build(&self, app: &mut App) {
//        let world = &mut app.world;
//        let s = world.query::<&Wall>();
//    }
//}

fn main() {
    App::new()
        .add_startup_system(setup)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameObjects {
            objects: HashMap::new(),
        })
        .add_plugins(DefaultPlugins)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1. / 15.))
                .with_system(keyboard_events),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(FPS as f64))
                .with_system(check_for_collision)
                .with_system(apply_gravity.before(check_for_collision)),
        )
        .run();
}

struct Square {
    pos_x: f32,
    pos_y: f32,
}

impl Square {
    fn new(x: f32, y: f32) -> Square {
        Square { pos_x: x, pos_y: y }
    }
}

struct Shape {
    squares: Vec<Square>,
}

#[derive(Debug)]
enum ShapeTypes {
    Square,
    Line,
    SquareTop,
    Zigzag,
    LShape,
}

impl Distribution<ShapeTypes> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ShapeTypes {
        match rng.gen_range(0..5) {
            0 => ShapeTypes::Square,
            1 => ShapeTypes::Line,
            2 => ShapeTypes::SquareTop,
            3 => ShapeTypes::Zigzag,
            4 => ShapeTypes::LShape,
            _ => unreachable!(),
        }
    }
}

const SQUARE_SIZE: f32 = 20.;

impl ShapeTypes {
    fn build(self, x: f32, y: f32) -> Shape {
        match self {
            ShapeTypes::Square => Shape {
                squares: vec![
                    Square::new(x, y),
                    Square::new(x + SQUARE_SIZE, y),
                    Square::new(x, y - SQUARE_SIZE),
                    Square::new(x + SQUARE_SIZE, y - SQUARE_SIZE),
                ],
            },
            ShapeTypes::Line => Shape {
                squares: vec![
                    Square::new(x, y),
                    Square::new(x + SQUARE_SIZE, y),
                    Square::new(x + SQUARE_SIZE + SQUARE_SIZE, y),
                    Square::new(x + SQUARE_SIZE + SQUARE_SIZE + SQUARE_SIZE, y),
                ],
            },
            ShapeTypes::SquareTop => Shape {
                squares: vec![
                    Square::new(x, y),
                    Square::new(x + SQUARE_SIZE, y),
                    Square::new(x + SQUARE_SIZE + SQUARE_SIZE, y),
                    Square::new(x + SQUARE_SIZE, y + SQUARE_SIZE),
                ],
            },
            ShapeTypes::Zigzag => Shape {
                squares: vec![
                    Square::new(x, y),
                    Square::new(x + SQUARE_SIZE, y),
                    Square::new(x + SQUARE_SIZE, y - SQUARE_SIZE),
                    Square::new(x + SQUARE_SIZE + SQUARE_SIZE, y - SQUARE_SIZE),
                ],
            },
            ShapeTypes::LShape => Shape {
                squares: vec![
                    Square::new(x, y),
                    Square::new(x + SQUARE_SIZE, y),
                    Square::new(x + SQUARE_SIZE + SQUARE_SIZE, y),
                    Square::new(x + SQUARE_SIZE + SQUARE_SIZE, y + SQUARE_SIZE),
                ],
            },
        }
    }
}

fn generate_random_color() -> Color {
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0..=255) as f32;
    let g = rng.gen_range(0..=255) as f32;
    let b = rng.gen_range(0..=255) as f32;

    return Color::rgb(r / 255., g / 255., b / 255.);
}

fn spawn_random_shape(commands: &mut Commands, game_objects: &mut GameObjects) {
    let mut entities: Vec<Entity> = Vec::new();
    let shape_type: ShapeTypes = rand::random();
    println!("{:?}", shape_type);

    let color = generate_random_color();
    let shape: Shape = shape_type.build(START_X, START_Y);
    for square in shape.squares {
        let entity = spawn_square(commands, color, square.pos_x, square.pos_y);
        entities.push(entity);
    }

    for entity in entities.clone() {
        game_objects.objects.insert(entity, entities.clone());
    }
}

fn spawn_square(commands: &mut Commands, color: Color, x: f32, y: f32) -> Entity {
    return commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(x, y, 0.0),
                scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.),
                ..default()
            },
            sprite: Sprite {
                color: color,
                ..default()
            },
            ..default()
        })
        .insert(Gravity::default())
        .insert(Collider)
        .id();
}

const START_X: f32 = -20.;
const START_Y: f32 = 180.;

fn setup(
    mut commands: Commands,
    mut game_objects: ResMut<GameObjects>,
    _asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn()
        .insert_bundle(WallBundle::new(WallLocation::Left))
        .insert(Collider)
        .insert(Wall)
        .insert(Block);
    commands
        .spawn()
        .insert_bundle(WallBundle::new(WallLocation::Right))
        .insert(Collider)
        .insert(Wall)
        .insert(Block);
    commands
        .spawn()
        .insert_bundle(WallBundle::new(WallLocation::Bottom))
        .insert(Collider)
        .insert(Wall)
        .insert(Block);
    commands
        .spawn()
        .insert_bundle(WallBundle::new(WallLocation::Top))
        .insert(Collider)
        .insert(Wall)
        .insert(Block);

    spawn_random_shape(&mut commands, &mut game_objects);
}

fn keyboard_events(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &Gravity), With<Collider>>,
    block_query: Query<(&Transform, &Block), Without<Gravity>>,
) {
    if keyboard_input.pressed(KeyCode::Left) {
        for (gravity_transform, _) in query.iter() {
            for (block_transform, _) in block_query.iter() {
                let block_transform_scale = block_transform.scale.truncate();
                let gravity_transform_scale = gravity_transform.scale.truncate();

                let c = collide(
                    block_transform.translation,
                    Vec2::new(block_transform_scale.x + 1., block_transform_scale.y + 1.),
                    gravity_transform.translation,
                    Vec2::new(
                        gravity_transform_scale.x + 1.,
                        gravity_transform_scale.y + 1.,
                    ),
                );

                if let Some(c) = c {
                    match c {
                        Collision::Left => return,
                        _ => {}
                    }
                }
            }
        }

        for (mut transform, _) in query.iter_mut() {
            transform.translation.x -= 20.;
        }
    }

    if keyboard_input.pressed(KeyCode::Right) {
        for (gravity_transform, _) in query.iter() {
            for (block_transform, _) in block_query.iter() {
                let block_transform_scale = block_transform.scale.truncate();
                let gravity_transform_scale = gravity_transform.scale.truncate();

                let c = collide(
                    block_transform.translation,
                    Vec2::new(block_transform_scale.x + 1., block_transform_scale.y + 1.),
                    gravity_transform.translation,
                    Vec2::new(
                        gravity_transform_scale.x + 1.,
                        gravity_transform_scale.y + 1.,
                    ),
                );

                if let Some(c) = c {
                    match c {
                        Collision::Right => return,
                        _ => {}
                    }
                }
            }
        }

        for (mut transform, _) in query.iter_mut() {
            transform.translation.x += 20.;
        }
    }

    if keyboard_input.pressed(KeyCode::Down) {
        for (gravity_transform, _) in query.iter() {
            for (block_transform, _) in block_query.iter() {
                let block_transform_scale = block_transform.scale.truncate();
                let gravity_transform_scale = gravity_transform.scale.truncate();

                let c = collide(
                    block_transform.translation,
                    Vec2::new(block_transform_scale.x + 1., block_transform_scale.y + 1.),
                    gravity_transform.translation,
                    Vec2::new(
                        gravity_transform_scale.x + 1.,
                        gravity_transform_scale.y + 1.,
                    ),
                );

                if let Some(c) = c {
                    match c {
                        Collision::Bottom => return,
                        _ => {}
                    }
                }
            }
        }

        for (mut transform, _) in query.iter_mut() {
            transform.translation.y -= 20.;
        }
    }

    if keyboard_input.pressed(KeyCode::Up) {
        // TODO: Update this logic
        // Maybe use hard-coded versions
        // Rotate the gravity transform to clockwise 90 degrees
        // and check if it collides with any other gravity transforms
        // If it does, don't rotate
        // If it doesn't, rotate
        // If it collides with a block, don't rotate
        let mut mid_x = 0.;
        let mut mid_y = 0.;
        for (transform, _) in query.iter_mut() {
            mid_x += transform.translation.x;
            mid_y += transform.translation.y;
        }
        mid_x /= 4.;
        mid_y /= 4.;
        // find the closest translations to the midpoint
        let mut closest_x = 0.;
        let mut closest_y = 0.;
        let mut closest_dist = std::f32::MAX;
        for (transform, _) in query.iter() {
            let dist =
                (transform.translation.x - mid_x).abs() + (transform.translation.y - mid_y).abs();
            if dist < closest_dist {
                closest_x = transform.translation.x;
                closest_y = transform.translation.y;
                closest_dist = dist;
            }
        }

        // check collision
        for (gravity_transform, _) in query.iter() {
            for (block_transform, _) in block_query.iter() {
                let x = gravity_transform.translation.y + closest_x - closest_y;
                let y = -gravity_transform.translation.x + closest_y + closest_x;
                let block_transform_scale = block_transform.scale.truncate();
                let c = collide(
                    block_transform.translation,
                    Vec2::new(block_transform_scale.x + 1., block_transform_scale.y + 1.),
                    Vec3::new(x, y, 0.),
                    Vec2::new(block_transform_scale.x + 1., block_transform_scale.y + 1.),
                );

                if c.is_some() {
                    return;
                }
            }
        }

        for (mut transform, _) in query.iter_mut() {
            let x = transform.translation.x;
            let y = transform.translation.y;
            transform.translation.x = y + closest_x - closest_y;
            transform.translation.y = -x + closest_y + closest_x;
        }
    }
}

fn when_object_landed(
    commands: &mut Commands,
    block_query: Query<(Entity, &Transform, &Block), Without<Wall>>,
    game_objects: &mut GameObjects,
    gravity_entity: Entity,
    gravity_transform: &Transform,
) {
    let mut entity_matrix: Vec<Vec<u32>> = vec![vec![u32::MAX; 11]; 22];
    let mut entity_map: HashMap<u32, Entity> = HashMap::new();
    for (entity, transform, _) in block_query.iter() {
        let id = entity.id();
        let x = (transform.translation.x + 100.) / 20.;
        let y = (transform.translation.y + 240.) / 20.;

        entity_map.insert(id, entity);
        entity_matrix[y as usize][x as usize] = id;
    }

    // Add new gravity entity to the map
    let x = (gravity_transform.translation.x + 100.) / 20.;
    let y = (gravity_transform.translation.y + 240.) / 20.;
    entity_map.insert(gravity_entity.id(), gravity_entity);
    entity_matrix[y as usize][x as usize] = gravity_entity.id();

    // Check rows to identify if any are full
    let mut target_rows_to_delete = Vec::new();
    let mut max_target_row = 0;
    for y in 0..22 {
        let mut full = true;
        for x in 0..11 {
            if entity_matrix[y][x] == u32::MAX {
                full = false;
                break;
            }
        }
        if full {
            target_rows_to_delete.push(y);
            max_target_row = y;
        }
    }

    if target_rows_to_delete.len() > 0 {
        // Delete the rows
        for target_row_to_delete in target_rows_to_delete {
            for x in 0..11 {
                let id = entity_matrix[target_row_to_delete][x];
                commands.entity(entity_map[&id]).despawn();

                let related_entities = game_objects.objects.get(&entity_map[&id]).unwrap().clone();
                for related_entity in related_entities {
                    let related_entities = game_objects.objects.get_mut(&related_entity).unwrap();
                    related_entities.remove(
                        related_entities
                            .iter()
                            .position(|entity| entity.id() == entity_map[&id].id())
                            .unwrap(),
                    );
                }

                game_objects.objects.remove(&entity_map[&id]);
                entity_matrix[target_row_to_delete][x] = u32::MAX;
            }
        }

        resize_all_objects(entity_matrix, max_target_row, block_query);
    }

    // resize every row above it
    // call recursively
}

fn resize_all_objects(
    mut entity_matrix: Vec<Vec<u32>>,
    last_row: usize,
    mut block_query: Query<(Entity, &Transform, &Block), Without<Wall>>,
) {
    let mut resize_info_map: HashMap<u32, f32> = HashMap::new();
    for y in last_row..entity_matrix.len() {
        for x in 0..entity_matrix[y].len() {
            let id = entity_matrix[y][x];
            // how much down you can go to
            let mut max_y = y;
            while max_y > 0 && entity_matrix[max_y - 1][x] == u32::MAX {
                max_y -= 1;
            }
            resize_info_map.insert(id, (y - max_y) as f32 * 20.);
            entity_matrix[max_y][x] = entity_matrix[y][x];
            entity_matrix[y][x] = u32::MAX;
        }
    }

    for (block_entity, mut block_transform, _) in block_query.iter_mut() {
        if let Some(resize_info) = resize_info_map.get(&block_entity.id()) {
            block_transform.translation.y -= *resize_info;
        }
    }
}

// TODO: Store each and every position in hashmap
// Whenever a block is moved check the hashmap if any collision happens
// If collision happens, then stop the block
fn check_for_collision(
    mut commands: Commands,
    mut game_objects: ResMut<GameObjects>,
    gravity_query: Query<(Entity, &Transform, &Gravity), With<Collider>>,
    block_query: Query<(Entity, &Transform, &Block), With<Collider>>,
    without_wall_query: Query<(Entity, &Transform, &Block), Without<Wall>>,
) {
    for (gravity_entity, gravity_transform, _) in gravity_query.iter() {
        for (_, block_transform, _) in block_query.iter() {
            let block_transform_scale = block_transform.scale.truncate();
            let gravity_transform_scale = gravity_transform.scale.truncate();

            let c = collide(
                block_transform.translation,
                Vec2::new(block_transform_scale.x + 1., block_transform_scale.y + 1.),
                gravity_transform.translation,
                Vec2::new(
                    gravity_transform_scale.x + 1.,
                    gravity_transform_scale.y + 1.,
                ),
            );

            if let Some(c) = c {
                match c {
                    Collision::Inside => {}
                    Collision::Bottom => {
                        remove_related_entities(&mut commands, &mut game_objects, gravity_entity);
                        spawn_random_shape(&mut commands, &mut game_objects);
                        when_object_landed(
                            &mut commands,
                            without_wall_query,
                            &mut game_objects,
                            gravity_entity,
                            gravity_transform,
                        );
                        return;
                    }
                    Collision::Left => {}
                    Collision::Right => {}
                    Collision::Top => {} // TODO: Gameover
                }
            }
        }
    }
}

fn apply_gravity(
    mut query: Query<(&mut Transform, &Gravity)>,
    block_query: Query<(&Transform, &Block), Without<Gravity>>,
) {
    for (gravity_transform, _) in query.iter() {
        for (block_transform, _) in block_query.iter() {
            let block_transform_scale = block_transform.scale.truncate();
            let gravity_transform_scale = gravity_transform.scale.truncate();

            let c = collide(
                block_transform.translation,
                Vec2::new(block_transform_scale.x + 1., block_transform_scale.y + 1.),
                gravity_transform.translation,
                Vec2::new(
                    gravity_transform_scale.x + 1.,
                    gravity_transform_scale.y + 1.,
                ),
            );

            if let Some(c) = c {
                match c {
                    Collision::Bottom => return,
                    _ => {}
                }
            }
        }
    }
    for (mut transform, gravity) in query.iter_mut() {
        transform.translation.y -= gravity.y;
    }
}

fn remove_related_entities(commands: &mut Commands, game_objects: &mut GameObjects, id: Entity) {
    let entities: Vec<Entity> = game_objects.objects.get(&id).unwrap().to_vec();
    for entity in entities {
        commands.entity(entity).insert(Block).remove::<Gravity>();
    }
}
