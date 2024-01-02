use bevy::prelude::*;

use crate::state::MainState;

use super::{
    record::{discretized_time, Record},
    update::BoardQuery,
};

/// Stores information about the state of the replay (i.e. paused or played, frames progressed).
#[derive(Resource)]
pub struct ReplayInfo {
    /// The current frame the replay occupies.
    frame: u64,
    /// Index into the most recently played item in the record.
    ix: usize,
    playing: Option<InitialReplayFrame>,
}

/// If the game is unpaused, this struct holds information about where the replay began, both in
/// engine time and in time with respect to the beginning of the current record.
pub struct InitialReplayFrame {
    /// The frame within the record from which replaying began.
    record_frame: u64,
    /// The engine time from which the replay was unpaused or started.
    real_frame: u64,
}

pub fn replay(record: Res<Record>, replay_info: Res<ReplayInfo>, mut board: Query<BoardQuery>) {
    let mut board = board.single_mut();

    record.data[replay_info.ix..]
        .iter()
        .take_while(|item| item.time == replay_info.frame)
        .for_each(|item| board.apply_record(item));
}

fn advance_frame(record: Res<Record>, mut replay_info: ResMut<ReplayInfo>, time: Res<Time>) {
    if let Some(initial) = &replay_info.playing {
        let current_time = discretized_time(&time);
        let elapsed_time = current_time - initial.real_frame;
        let new_record_frame = initial.record_frame + elapsed_time;
        if new_record_frame != replay_info.frame {
            replay_info.frame = new_record_frame;
        }
    }
}

pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            replay.run_if(in_state(MainState::PostGame).and_then(resource_changed::<ReplayInfo>())),
        )
        .add_systems(
            PostUpdate,
            advance_frame.run_if(in_state(MainState::PostGame)),
        );
    }
}
