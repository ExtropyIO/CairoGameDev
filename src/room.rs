use crate::character::Player;
use crate::resources::*;
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
        Name::new("painting"),
    ));
}

fn highlight_object(
    mut commands: Commands,
    mut objects: Query<((Entity, &Transform, &Handle<Image>, &Name), With<Object>)>,
    mut characters: Query<(&Transform, &Player)>,
    assets: Res<Assets<Image>>,
    input: Res<Input<KeyCode>>,
    interact_object: Res<InteractObjectState>,
    escape_action: Res<EscapeState>,
    mut evr_char: EventReader<ReceivedCharacter>,
    kbd: Res<Input<KeyCode>>,
    mut string: Local<String>,
) {
    let character_transform = characters.single_mut();

    for ((object_entity, object_transform, handle, obj_name), mut object) in &mut objects {
        let image_size = assets
            .get(handle)
            .map(|result| result.size())
            .unwrap_or(Vec2::new(0.0, 0.0));
        // 0.25 because we divide x by 2 and then take the scale factor 0.5
        let object_min = object_transform.translation.x - image_size.x * 0.25;
        let object_max = object_transform.translation.x + image_size.x * 0.25;

        let character_x = character_transform.0.translation.x;

        if character_x > object_min && character_x < object_max {
            if input.just_pressed(KeyCode::E) {
                if obj_name.to_string() == "Door" {
                    println!("The secret to open the door is: {}", &*string);
                    if let Err(e) = escape_action.try_send(string.to_string()) {
                        log::error!("Escpae state channel: {e}");
                    }
                    return;
                }

                if let Err(e) = interact_object.try_send(obj_name.to_string()) {
                    log::error!("Interact object channel: {e}");
                }
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
    for ev in evr_char.iter() {
        // ignore control (special) characters
        if !ev.char.is_control() {
            string.push(ev.char);
        }
    }
}
