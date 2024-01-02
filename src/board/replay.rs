use bevy::prelude::*;

use super::{record::Record, update::BoardQuery};

/// Stores information about the state of the replay (i.e. paused or played, frames progressed).
#[derive(Resource)]
struct ReplayInfo {
    /// The current frame the replay occupies.
    frame: u64,
    /// Index into the most recently played item in the record.
    ix: usize,
    playing: Option<InitialReplayFrame>,
}

/// If the game is unpaused, this struct holds information about where the replay began, both in
/// engine time and in time with respect to the beginning of the current record.
struct InitialReplayFrame {
    /// The frame within the record from which replaying began.
    record_frame: u64,
    /// The engine time from which the replay was unpaused or started.
    real_frame: u64,
}

pub fn replay(record: Res<Record>, replay_info: Res<ReplayInfo>, board: Query<BoardQuery>) {
    unimplemented!()
}

fn advance_frame(record: Res<Record>, mut replay_info: ResMut<ReplayInfo>, time: Res<Time>) {
    unimplemented!()
}
