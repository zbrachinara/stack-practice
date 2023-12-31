//! Replay code currently depends on the board being unique in the world.

use bevy::prelude::*;

use crate::state::MainState;

use super::{
    record::{discretized_time, Record},
    update::BoardQuery,
};

/// Stores information about the state of the replay (i.e. paused or played, frames progressed).
#[derive(Resource, Default)]
pub struct ReplayInfo {
    /// The current frame the replay occupies.
    frame: u64,
    /// Index into the most recently played item in the record.
    ix: usize,
    playing: Option<InitialReplayFrame>,
}

/// If the game is unpaused, this struct holds information about where the replay began, both in
/// engine time and in time with respect to the beginning of the current record.
#[derive(Debug)]
pub struct InitialReplayFrame {
    /// The frame within the record from which replaying began.
    record_frame: u64,
    /// The engine time from which the replay was unpaused or started.
    real_frame: u64,
}

pub fn initialize_replay(mut commands: Commands, mut board: Query<BoardQuery>) {
    board.single_mut().clear_board();
    commands.insert_resource(ReplayInfo::default());
}

pub fn cleanup_replay(mut commands: Commands) {
    commands.remove_resource::<ReplayInfo>()
}

pub fn replay(record: Res<Record>, replay_info: Res<ReplayInfo>, mut board: Query<BoardQuery>) {
    let mut board = board.single_mut();

    // TODO also account for frame skips
    record.data[replay_info.ix..]
        .iter()
        .filter(|item| item.time == replay_info.frame)
        .for_each(|item| board.apply_record(item));
}

fn advance_frame(mut replay_info: ResMut<ReplayInfo>, time: Res<Time>) {
    if let Some(initial) = &replay_info.playing {
        let current_time = discretized_time(&time);
        let elapsed_time = current_time - initial.real_frame;
        let new_record_frame = initial.record_frame + elapsed_time;
        if new_record_frame != replay_info.frame {
            // TODO also advance ix
            replay_info.frame = new_record_frame;
        }
    }
}

fn toggle_pause(mut replay_info: ResMut<ReplayInfo>, input: Res<Input<KeyCode>>, time: Res<Time>) {
    if input.just_pressed(KeyCode::Space) {
        if replay_info.playing.is_some() {
            replay_info.playing = None;
        } else {
            let real_time = discretized_time(&time);
            let frame_time = replay_info.frame;

            replay_info.playing = Some(InitialReplayFrame {
                record_frame: frame_time,
                real_frame: real_time,
            });
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
        .add_systems(Update, toggle_pause.run_if(in_state(MainState::PostGame)))
        .add_systems(
            PostUpdate,
            advance_frame.run_if(in_state(MainState::PostGame)),
        )
        .add_systems(OnEnter(MainState::PostGame), initialize_replay)
        .add_systems(OnExit(MainState::PostGame), cleanup_replay);
    }
}
