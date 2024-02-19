use crate::replay::record::{record, CompleteRecord, FirstFrame, PartialRecord};
use crate::replay::replay::{replay, DeferUnfreeze, ReplayInfo};
use crate::state::MainState;
use crate::{board, controller};
use bevy::prelude::*;

pub mod record;
pub mod replay;

pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompleteRecord>()
            .init_resource::<PartialRecord>()
            .add_event::<DeferUnfreeze>()
            .add_systems(
                Update,
                replay.run_if(
                    in_state(MainState::PostGame).and_then(resource_changed::<ReplayInfo>),
                ),
            )
            .add_systems(
                PostUpdate,
                record
                    .run_if(resource_exists::<FirstFrame>.and_then(in_state(MainState::Playing))),
            )
            .add_systems(
                PostUpdate,
                (
                    replay::adjust_replay,
                    replay::advance_frame,
                    replay::update_progress,
                    replay::exit_replay.before(controller::reset_controller),
                )
                    .chain()
                    .run_if(in_state(MainState::PostGame)),
            )
            .add_systems(OnExit(MainState::Playing), record::finalize_record)
            // systems which run when starting a clean record
            .add_systems(
                OnTransition {
                    from: MainState::PostGame,
                    to: MainState::Ready,
                },
                record::reset_record,
            )
            .add_systems(
                OnTransition {
                    from: MainState::Ready,
                    to: MainState::Playing,
                },
                record::initialize_time,
            )
            // systems which run when beginning a new segment into a record
            .add_systems(
                OnTransition {
                    from: MainState::PostGame,
                    to: MainState::Playing,
                },
                record::begin_new_segment,
            )
            .add_systems(
                Update,
                replay::unfreeze_controller_after_exit
                    .run_if(on_event::<DeferUnfreeze>())
                    .after(board::update::update_board),
            )
            // common systems which run on each entrance into/exit from replay
            .add_systems(
                OnEnter(MainState::PostGame),
                (replay::initialize_replay, replay::setup_progress_bar),
            )
            .add_systems(
                OnExit(MainState::PostGame),
                (replay::cleanup_replay, replay::remove_progress_bar),
            );
    }
}
