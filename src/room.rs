use crate::character::Player;
use crate::resources::InteractObjectState;
use crate::resources::ObjectNameInteraction;
use bevy::log;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_inspector_egui::InspectorOptions;
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // loading the asset

    let texture = asset_server.load("object_bookcase.png");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(-40.0, -40.0, 0.0).with_scale(Vec3::splat(0.5)),
            texture,
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            ..default()
        },
        Object {
            name: "Bookcase".to_string(),
        },
        Name::new("Bookcase"),
    ));

    let texture = asset_server.load("object_cupboard.png");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(35.0, -40.0, 0.0).with_scale(Vec3::splat(0.5)),
            texture,
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            ..default()
        },
        Object {
            name: "Cupboard".to_string(),
        },
        Name::new("Cupboard"),
    ));

    let texture = asset_server.load("object_door.png");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(125.0, -40.0, 0.0).with_scale(Vec3::splat(0.5)),
            texture,
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            ..default()
        },
        Object {
            name: "Door".to_string(),
        },
        Name::new("Door"),
    ));

    let texture = asset_server.load("object_table.png");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(-110.0, -40.0, 0.0).with_scale(Vec3::splat(0.5)),
            texture,
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            ..default()
        },
        Object {
            name: "Table".to_string(),
        },
        Name::new("Table"),
    ));

    let texture = asset_server.load("object_window.png");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(80.0, -12.50, 0.0).with_scale(Vec3::splat(0.5)),
            texture,
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            ..default()
        },
        Object {
            name: "Window".to_string(),
        },
        Name::new("Window"),
    ));

    let texture = asset_server.load("object_painting.png");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(4.0, -12.50, 0.0).with_scale(Vec3::splat(0.5)),
            texture,
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            ..default()
        },
        Object {
            name: "Painting".to_string(),
        },
        Name::new("Painting"),
    ));
}

fn highlight_object(
    mut commands: Commands,
    mut objects: Query<((Entity, &Transform, &Handle<Image>, &Name), With<Object>)>,
    mut characters: Query<(&Transform, &Player)>,
    assets: Res<Assets<Image>>,
    input: Res<Input<KeyCode>>,
    interact_object: Res<InteractObjectState>,
    // mut object_name: ResMut<ObjectNameInteraction>,
) {
    let character_transform = characters.single_mut();

    for ((object_entity, object_transform, handle, obj_name), mut object) in &mut objects {
        let image = assets.get(handle).unwrap();
        let image_size = image.size();

        let object_min = object_transform.translation.x - image_size.x * 0.25;
        let object_max = object_transform.translation.x + image_size.x * 0.25;

        let character_x = character_transform.0.translation.x;

        if character_x > object_min && character_x < object_max {
            if input.just_pressed(KeyCode::E) {
                // check if object is door:
                if obj_name.to_string() == "Door" {
                    println!("This is door");
                }
                println!("Interaction key: {}", obj_name.to_string());
                // object_name.0 = obj_name.to_string();
                println!("{}, {}", object_min, object_max);
                println!("Interacted with the object.");
                if let Err(e) = interact_object.try_send(obj_name.to_string()) {
                    log::error!("Spawn players channel: {e}");
                }
            }
        }
    }
}
