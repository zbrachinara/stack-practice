use bevy::{
    app::{Plugin, PostUpdate, Update},
    ecs::{
        query::{Added, Changed, Or},
        schedule::{common_conditions::in_state, IntoSystemConfigs},
    },
    sprite::Material2dPlugin,
};

use crate::state::MainState;

use self::{
    active::display_active,
    floor::{spawn_drop_shadow, update_drop_shadow, DropShadowMaterial},
    hold::display_held,
    matrix::{center_board, redraw_board},
    queue::display_queue,
};

type AddedOrChanged<T> = Or<(Added<T>, Changed<T>)>;

mod active;
mod floor;
mod hold;
mod matrix;
mod queue;

pub struct BoardDisplayPlugin;

impl Plugin for BoardDisplayPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(Material2dPlugin::<DropShadowMaterial>::default())
            .add_systems(Update, spawn_drop_shadow)
            .add_systems(
                PostUpdate,
                (
                    update_drop_shadow,
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
