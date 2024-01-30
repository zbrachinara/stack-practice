use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::transform::TransformSystem;

use crate::state::MainState;

use self::active::spawn_active_sprite;
use self::hold::spawn_hold_sprite;
use self::matrix::spawn_matrix_sprite;
use self::queue::spawn_queue_sprite;
use self::{
    active::display_active,
    floor::{spawn_drop_shadow, update_drop_shadow, DropShadowMaterial},
    hold::display_held,
    matrix::{center_board, redraw_board},
    queue::display_queue,
};

mod active;
mod floor;
mod hold;
mod matrix;
mod queue;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum DisplayEntitySet {
    Spawn,
    /// Spawning doesn't have any immediate effect unless the Command buffers are applied, so we
    /// apply the command buffers before moving to update the objects.
    ApplyBuffers,
    Update,
}

pub struct BoardDisplayPlugin;

impl Plugin for BoardDisplayPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(Material2dPlugin::<DropShadowMaterial>::default())
            .add_systems(
                PostUpdate,
                (
                    spawn_drop_shadow,
                    spawn_matrix_sprite,
                    spawn_active_sprite,
                    spawn_queue_sprite,
                    spawn_hold_sprite,
                )
                    .in_set(DisplayEntitySet::Spawn)
                    .before(DisplayEntitySet::ApplyBuffers)
                    .run_if(not(in_state(MainState::Loading))),
            )
            .add_systems(
                PostUpdate,
                apply_deferred.in_set(DisplayEntitySet::ApplyBuffers),
            )
            .add_systems(
                PostUpdate,
                (
                    update_drop_shadow,
                    center_board,
                    redraw_board,
                    display_active,
                    display_queue,
                    display_held,
                )
                    .in_set(DisplayEntitySet::Update)
                    .after(DisplayEntitySet::ApplyBuffers)
                    .before(TransformSystem::TransformPropagate)
                    .run_if(not(in_state(MainState::Loading))),
            );
    }
}
