use bevy::{
    input::common_conditions::input_toggle_active, prelude::*, render::camera::ScalingMode,
};
use bevy_asset_loader::loading_state::LoadingStateAppExt;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::{egui::Key, quick::WorldInspectorPlugin, InspectorOptions};
use game_demo::{
    character::CharacterPlugin, configs::*, dojo_new::DojoPlugin, resources::*, room::RoomPlugin,
    ui::GameUI,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Escape from Cairo".into(),
                        resolution: (640.0, 320.0).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .insert_resource(MovesRemaining(10.0))
        .register_type::<MovesRemaining>()
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .add_plugins(GameUI)
        .add_plugins(CharacterPlugin)
        .add_plugins(RoomPlugin)
        .add_plugins(DojoPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera = Camera2dBundle::default();

    // let's to have reasonable game coords
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 256.0,
        min_height: 144.0,
    };

    commands.spawn(camera);

    spawn_room(&mut commands, &asset_server);
}

fn spawn_room(commands: &mut Commands, asset_server: &AssetServer) {
    let texture = asset_server.load("room_background.png");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, -10.0).with_scale(Vec3::splat(0.5)),
            texture,
            ..default()
        },
        Name::new("Background"),
    ));
}
