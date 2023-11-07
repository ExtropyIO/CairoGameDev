use crate::character::Player;
use crate::dojo::{task_escape, task_interact, task_spawn_object, DojoEnv};
use crate::resources::*;
use bevy::{prelude::*, sprite::*};
use bevy_inspector_egui::InspectorOptions;
use starknet::core::{types::FieldElement, utils::cairo_short_string_to_felt};

pub struct RoomPlugin;
pub struct SpawnRoom;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ObjectNameInteraction(String::from("")))
            .add_systems(Startup, setup)
            .add_systems(Update, highlight_object);
    }
}
#[derive(Component, InspectorOptions, Default, Reflect)]
#[reflect(Component)]

pub struct Object {
    pub name: String,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, env: Res<DojoEnv>) {
    // loading the assets
    // TODO: Load it as a SpriteBundle
    let bookcase_texture = asset_server.load("object_bookcase.png");
    let cupboard_texture = asset_server.load("object_cupboard.png");
    let door_texture = asset_server.load("object_door.png");
    let table_texture = asset_server.load("object_table.png");
    let window_texture = asset_server.load("object_window.png");
    let painting_texture = asset_server.load("object_painting.png");

    // create a vector of tuples containing the texture and the transform for each object
    let objects = vec![
        (
            bookcase_texture,
            Transform::from_xyz(-40.0, -40.0, 0.0),
            "Bookcase",
            "A strange book, 1984",
        ),
        (
            cupboard_texture,
            Transform::from_xyz(35.0, -40.0, 0.0),
            "Cupboard",
            "An egyptian cat.",
        ),
        (
            door_texture,
            Transform::from_xyz(125.0, -40.0, 0.0),
            "Door",
            "Needs a key",
        ),
        (
            table_texture,
            Transform::from_xyz(-110.0, -40.0, 0.0),
            "Table",
            "Pile of papers.",
        ),
        (
            window_texture,
            Transform::from_xyz(80.0, -12.50, 0.0),
            "Window",
            "Raining outside...",
        ),
        (
            painting_texture,
            Transform::from_xyz(4.0, -12.50, 0.0),
            "Painting",
            "An intriguing painting.",
        ),
    ];

    // spawn a batch of entities with the same components
    commands.spawn_batch(
        objects
            .clone()
            .into_iter()
            .map(|(texture, transform, name, _)| {
                (
                    SpriteBundle {
                        transform: transform.with_scale(Vec3::splat(0.5)),
                        texture,
                        sprite: Sprite {
                            anchor: Anchor::BottomCenter,
                            ..default()
                        },
                        ..default()
                    },
                    Object {
                        name: name.to_string(),
                    },
                    Name::new(name),
                )
            }),
    );

    // Spawn each object on the dojo side.

    // Create a new vector with only the last two elements of each tuple
    let objects_data: Vec<(FieldElement, FieldElement)> = objects
        .iter()
        .map(|&(_, _, a, b)| {
            (
                cairo_short_string_to_felt(a).unwrap(),
                cairo_short_string_to_felt(b).unwrap(),
            )
        })
        .collect();

    // Separate objects_ids and objects_descriptions vectors
    let objects_ids: Vec<FieldElement> = objects_data.iter().map(|(id, _)| id.clone()).collect();
    let objects_descriptions: Vec<FieldElement> = objects_data
        .iter()
        .map(|(_, description)| description.clone())
        .collect();

    task_spawn_object(&mut commands, &env, objects_ids, objects_descriptions);
}

fn highlight_object(
    mut commands: Commands,
    mut objects: Query<((Entity, &Transform, &Handle<Image>, &Name), With<Object>)>,
    mut characters: Query<(&Transform, &Player)>,
    assets: Res<Assets<Image>>,
    input: Res<Input<KeyCode>>,
    mut evr_char: EventReader<ReceivedCharacter>,
    kbd: Res<Input<KeyCode>>,
    mut string: Local<String>,
    env: Res<DojoEnv>,
) {
    let character_transform = characters.single_mut();

    for ((_, object_transform, handle, obj_name), _) in &mut objects {
        let image_size = assets
            .get(handle)
            .map(|result| result.size())
            .unwrap_or(UVec2::new(0, 0));
        let image_size = Vec2::new(image_size.x as f32, image_size.y as f32);
        // 0.25 because we divide x by 2 and then take the scale factor 0.5
        let object_min = object_transform.translation.x - image_size.x * 0.25;
        let object_max = object_transform.translation.x + image_size.x * 0.25;

        let character_x = character_transform.0.translation.x;

        if character_x > object_min && character_x < object_max {
            if input.just_pressed(KeyCode::E) {
                if obj_name.to_string() == "Door" {
                    println!("The secret to open the door is: {}", &*string);
                    task_escape(&mut commands, &env, string.to_string());
                    return;
                }
                task_interact(
                    &mut commands,
                    &env,
                    cairo_short_string_to_felt(obj_name).unwrap(),
                );
            }
        }
    }

    if kbd.just_pressed(KeyCode::Return) {
        println!("Text input: {}", &*string);
        string.clear();
    }
    if kbd.just_pressed(KeyCode::Back) {
        string.pop();
    }
    for ev in evr_char.read() {
        // ignore control (special) characters
        if !ev.char.is_control() {
            string.push(ev.char);
        }
    }
}
