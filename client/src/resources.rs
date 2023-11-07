use bevy::prelude::*;

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct MovesRemaining(pub u64);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ObjectNameInteraction(pub String);
