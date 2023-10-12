use bevy::prelude::*;
use tokio::sync::mpsc;

#[derive(Resource)]
pub struct MovesRemaining(pub f32);

#[derive(Resource)]
pub struct StartGameCommand(pub mpsc::Sender<()>);

// TODO: derive macro?
impl StartGameCommand {
    pub fn try_send(&self) -> Result<(), mpsc::error::TrySendError<()>> {
        self.0.try_send(())
    }
}

#[derive(Resource)]
pub struct InteractObjectState(pub mpsc::Sender<()>);

impl InteractObjectState {
    pub fn try_send(&self) -> Result<(), mpsc::error::TrySendError<()>> {
        self.0.try_send(())
    }
}
