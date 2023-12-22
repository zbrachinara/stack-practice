use bevy::{app::Plugin, ecs::schedule::States};

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum MainState {
    #[default]
    Loading,
    Playing,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state::<MainState>();
    }
}