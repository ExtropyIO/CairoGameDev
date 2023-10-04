use crate::MovesRemaining;
use bevy::prelude::*;
pub struct GameUI;

#[derive(Component)]
pub struct MovesRemainingText;

impl Plugin for GameUI {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_game_ui)
            .add_systems(Update, update_remaining_moves);
    }
}

fn spawn_game_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(10.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                ..default()
            },
            Name::new("UI Root"),
        ))
        .with_children(|commands| {
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Moves remaining:",
                        TextStyle {
                            font_size: 32.0,
                            ..default()
                        },
                    ),
                    ..default()
                },
                MovesRemainingText,
            ));
        });
}

fn update_remaining_moves(
    mut texts: Query<&mut Text, With<MovesRemainingText>>,
    moves: Res<MovesRemaining>,
) {
    for mut text in &mut texts {
        text.sections[0].value = format!("Moves remaining: {:?}", moves.0);
    }
}
