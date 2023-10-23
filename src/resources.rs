use bevy::prelude::*;
use tokio::sync::mpsc;

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct MovesRemaining(pub f32);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ObjectNameInteraction(pub String);

#[derive(Resource)]
pub struct StartGameCommand(pub mpsc::Sender<()>);

impl StartGameCommand {
    pub fn try_send(&self) -> Result<(), mpsc::error::TrySendError<()>> {
        self.0.try_send(())
    }
}

#[derive(Resource)]
pub struct EscapeState(pub mpsc::Sender<String>);

impl EscapeState {
    pub fn try_send(&self, data: String) -> Result<(), mpsc::error::TrySendError<String>> {
        self.0.try_send(data)
    }
}

#[derive(Resource)]
pub struct InteractObjectState(pub mpsc::Sender<String>);

impl InteractObjectState {
    pub fn try_send(&self, data: String) -> Result<(), mpsc::error::TrySendError<String>> {
        self.0.try_send(data)
    }
}
