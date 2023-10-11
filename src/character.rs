use bevy_inspector_egui::InspectorOptions;

use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (animate_sprite, character_movement))
            .register_type::<Player>(); // for new types
    }
}
#[derive(Component, InspectorOptions, Default, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub speed: f32,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("duck_idle_2.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(28.0, 28.0), 6, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let animation_indices = AnimationIndices { first: 1, last: 5 };
    commands.spawn((
        SpriteSheetBundle {
            transform: Transform::from_xyz(0.0, -27.5, 10.0),
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        Name::new("Player"),
        Player { speed: 100.0 },
    ));
}

fn character_movement(
    mut characters: Query<(&mut Transform, &Player)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, player) in &mut characters {
        let movement_amount = player.speed * time.delta_seconds();

        if input.pressed(KeyCode::A) {
            transform.translation.x -= movement_amount;
            // set walking image
        }
        if input.pressed(KeyCode::D) {
            transform.translation.x += movement_amount;
            // set walking image
        }
    }
}
