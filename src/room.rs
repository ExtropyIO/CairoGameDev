use crate::character::Player;
use crate::resources::*;
use bevy::{log, prelude::*, sprite::*};
use bevy_inspector_egui::InspectorOptions;

pub struct RoomPlugin;
pub struct SpawnRoom;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ObjectNameInteraction(String::from("")))
            .add_systems(Startup, setup);
        // .add_systems(Update, highlight_object);
    }
}
#[derive(Component, InspectorOptions, Default, Reflect)]
#[reflect(Component)]
pub struct Object {
    pub name: String,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    commands.spawn_batch(objects.into_iter().map(|(texture, transform, name, _)| {
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
    }));

    // Spawn each object on the dojo side.

    // let mut iterator = objects.iter();
    // while let Some((_, _, name, description)) = iterator.next() {
    //     // do something with texture, transform, and name
    //     if let Err(e) = spawn_object.try_send((name.to_string(), description.to_string())) {
    //         log::error!("Interact object channel: {e}");
    //     }
    // }
}

// fn highlight_object(
//     mut commands: Commands,
//     mut objects: Query<((Entity, &Transform, &Handle<Image>, &Name), With<Object>)>,
//     mut characters: Query<(&Transform, &Player)>,
//     assets: Res<Assets<Image>>,
//     input: Res<Input<KeyCode>>,
//     interact_object: Res<InteractObjectState>,
//     escape_action: Res<EscapeState>,
//     mut evr_char: EventReader<ReceivedCharacter>,
//     kbd: Res<Input<KeyCode>>,
//     mut string: Local<String>,
// ) {
//     let character_transform = characters.single_mut();

//     for ((object_entity, object_transform, handle, obj_name), mut object) in &mut objects {
//         let image_size = assets
//             .get(handle)
//             .map(|result| result.size())
//             .unwrap_or(Vec2::new(0.0, 0.0));
//         // 0.25 because we divide x by 2 and then take the scale factor 0.5
//         let object_min = object_transform.translation.x - image_size.x * 0.25;
//         let object_max = object_transform.translation.x + image_size.x * 0.25;

//         let character_x = character_transform.0.translation.x;

//         if character_x > object_min && character_x < object_max {
//             if input.just_pressed(KeyCode::E) {
//                 if obj_name.to_string() == "Door" {
//                     println!("The secret to open the door is: {}", &*string);
//                     if let Err(e) = escape_action.try_send(string.to_string()) {
//                         log::error!("Escpae state channel: {e}");
//                     }
//                     return;
//                 }

//                 if let Err(e) = interact_object.try_send(obj_name.to_string()) {
//                     log::error!("Interact object channel: {e}");
//                 }
//             }
//         }
//     }

//     if kbd.just_pressed(KeyCode::Return) {
//         println!("Text input: {}", &*string);
//         string.clear();
//     }
//     if kbd.just_pressed(KeyCode::Back) {
//         string.pop();
//     }
//     for ev in evr_char.iter() {
//         // ignore control (special) characters
//         if !ev.char.is_control() {
//             string.push(ev.char);
//         }
//     }
// }
