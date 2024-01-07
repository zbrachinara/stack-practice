use bevy::prelude::*;

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum MainState {
    #[default]
    Loading,
    Ready,
    Playing,
    PostGame,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<MainState>();
    }
}
