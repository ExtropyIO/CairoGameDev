use bevy::prelude::*;
use bevy::sprite::*;
pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
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
        Name::new("Object_Bookcase"),
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
        Name::new("Object_Cupboard"),
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
        Name::new("Object_Door"),
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
        Name::new("Object_Table"),
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
        Name::new("Object_Window"),
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
        Name::new("Object_Painting"),
    ));
}
