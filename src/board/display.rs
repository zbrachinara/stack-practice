use bevy::{
    app::{Plugin, PostUpdate},
    ecs::{
        query::{Added, Changed, Or},
        schedule::{common_conditions::in_state, IntoSystemConfigs},
    },
};

use crate::state::MainState;

use self::{
    active::display_active,
    hold::display_held,
    matrix::{center_board, redraw_board},
    queue::display_queue,
};

type AddedOrChanged<T> = Or<(Added<T>, Changed<T>)>;

mod active;
mod hold;
mod matrix;
mod queue;

pub struct BoardDisplayPlugin;

impl Plugin for BoardDisplayPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PostUpdate,
            (
                center_board,
                display_active,
                display_queue,
                display_held,
                redraw_board,
            )
                .run_if(in_state(MainState::Playing)),
        );
    }
}
