use crate::replay::record::{record, CompleteRecord, FirstFrame, PartialRecord};
use crate::replay::replay::{replay, ReplayInfo};
use crate::state::MainState;
use bevy::prelude::*;

pub mod record;
pub mod replay;

pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompleteRecord>()
            .init_resource::<PartialRecord>()
            .add_systems(
                Update,
                replay.run_if(
                    in_state(MainState::PostGame).and_then(resource_changed::<ReplayInfo>()),
                ),
            )
            .add_systems(
                PostUpdate,
                record
                    .run_if(resource_exists::<FirstFrame>().and_then(in_state(MainState::Playing))),
            )
            .add_systems(
                PostUpdate,
                (
                    replay::adjust_replay,
                    replay::advance_frame,
                    replay::update_progress,
                )
                    .chain()
                    .run_if(in_state(MainState::PostGame)),
            )
            .add_systems(OnExit(MainState::Playing), record::finalize_record)
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
