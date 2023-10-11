use crate::character::Player;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_inspector_egui::InspectorOptions;
pub struct RoomPlugin;
pub struct SpawnRoom;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
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
        Name::new("Object"),
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
        Name::new("Object"),
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
        Name::new("Object"),
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
        Name::new("Object"),
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
        Name::new("Object"),
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
        Name::new("Object"),
    ));
}

fn highlight_object(
    mut commands: Commands,
    mut objects: Query<((Entity, &Transform), With<Object>)>,
    mut characters: Query<(&Transform, &Player)>,
    input: Res<Input<KeyCode>>,
) {
    let character_transform = characters.single_mut();

    for ((object_entity, object_transform), mut object) in &mut objects {
        let object_size = object_transform.scale.truncate();

        // TODO: check the length is actual
        let object_min = object_transform.translation.x - object_size.x;
        let object_max = object_transform.translation.x + object_size.x;

        let character_x = character_transform.0.translation.x;

        if character_x > object_min && character_x < object_max {
            if input.pressed(KeyCode::E) {
                println!("Interacted with the object.")
            }
            println!("Object is near player");
            println!("{}", object_min);
            println!("{}", object_max);
            // object_entity.apply(value)
            // commands.entity(object_entity).add(Color::WHITE);
        }
    }
}
