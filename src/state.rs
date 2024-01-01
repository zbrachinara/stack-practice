use bevy::{
    app::{Plugin, Update},
    ecs::{
        schedule::{common_conditions::in_state, Condition, IntoSystemConfigs, NextState, States},
        system::{Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
};

use crate::{
    board::Settings,
    screens::{apply_settings, GlobalSettings},
};

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
        app.add_state::<MainState>().add_systems(
            Update,
            start_playing
                .run_if(in_state(MainState::Ready).or_else(in_state(MainState::PostGame)))
                .after(apply_settings),
        );
    }
}

fn start_playing(
    input: Res<Input<KeyCode>>,
    mut state: ResMut<NextState<MainState>>,
    settings: Res<GlobalSettings>,
) {
    if input.just_pressed(KeyCode::Grave) && Settings::try_from(&*settings).is_ok() {
        state.0 = Some(MainState::Playing);
    }
}
