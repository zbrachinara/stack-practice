use bevy::prelude::*;
use bevy::{ecs::schedule::ScheduleLabel, sprite::Material2dPlugin};

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

#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone)]
pub struct SpawnDisplayEntities;
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone)]
pub struct UpdateDisplayEntities;

fn display_schedule(world: &mut World) {
    world.run_schedule(SpawnDisplayEntities);
    world.run_schedule(UpdateDisplayEntities);
}

pub struct BoardDisplayPlugin;

impl Plugin for BoardDisplayPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(Material2dPlugin::<DropShadowMaterial>::default())
            .add_systems(PostUpdate, display_schedule)
            .add_systems(SpawnDisplayEntities, spawn_drop_shadow)
            .add_systems(
                UpdateDisplayEntities,
                (
                    update_drop_shadow,
                    center_board,
                    display_active,
                    display_queue,
                    display_held,
                    redraw_board,
                )
                    .run_if(not(in_state(MainState::Loading))),
            );
    }
}
