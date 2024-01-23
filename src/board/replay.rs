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
    /// Index of the item after the most recently played item in the record. If the replay is being reversed, this is
    /// the index of the most recently undone item in the record (no matter the current direction of time, these are the
    /// same thing).
    ix: usize,
    /// The index which `ix` needs to reach in order to be on time.
    next_ix: usize,
    playing: Option<ActiveReplayMeta>,
}

/// If the game is unpaused, this struct holds metadata about how the replay should be reading the record.
#[derive(Debug, Clone, Copy)]
pub struct ActiveReplayMeta {
    /// The frame within the record from which replaying began.
    record_frame: u64,
    /// The engine time from which the replay was unpaused or started.
    real_frame: u64,
    /// The current replay's direction through time (`false` is forward, `true` is backward).
    reverse: bool,
}

pub fn initialize_replay(mut commands: Commands, mut board: Query<BoardQuery>) {
    board.single_mut().clear_board();
    commands.init_resource::<ReplayInfo>();
}

pub fn cleanup_replay(mut commands: Commands) {
    commands.remove_resource::<ReplayInfo>();
    commands.init_resource::<Record>()
}

pub fn replay(
    record: Res<Record>,
    mut replay_info: ResMut<ReplayInfo>,
    mut board: Query<BoardQuery>,
) {
    let mut board = board.single_mut();
    if let Some(meta) = replay_info.playing {
        if meta.reverse {
            for item in record.data[replay_info.next_ix..replay_info.ix]
                .iter()
                .rev()
            {
                board.undo_record(item);
            }
        } else {
            for item in &record.data[replay_info.ix..replay_info.next_ix] {
                board.apply_record(item);
            }
        }
    }
    // if replay_info.playing.is_some_and(|meta| meta.reverse) {
    // } else if replay_info.playing.is_some() {
    // }
    replay_info.ix = replay_info.next_ix;
}

fn advance_frame(mut replay_info: ResMut<ReplayInfo>, record: Res<Record>, time: Res<Time>) {
    if let Some(initial) = replay_info.playing {
        let current_time = discretized_time(&time);
        let elapsed_time = current_time - initial.real_frame;

        let new_record_frame = if initial.reverse {
            initial.record_frame.saturating_sub(elapsed_time)
        } else {
            initial.record_frame + elapsed_time
        };

        if new_record_frame != replay_info.frame {
            replay_info.frame = new_record_frame;

            replay_info.next_ix = if initial.reverse {
                record.data[..=std::cmp::min(replay_info.ix, record.data.len() - 1)]
                    .iter()
                    .rev()
                    .position(|item| item.time < new_record_frame)
                    .map(|ix| replay_info.ix - ix + 1)
                    .unwrap_or(0)
            } else {
                record.data[replay_info.ix..]
                    .iter()
                    .position(|item| item.time > new_record_frame)
                    .map(|ix| replay_info.ix + ix)
                    .unwrap_or(record.data.len())
            };
        }

        // pause replay after reaching the end of the record
        if (replay_info.ix == record.data.len() && !initial.reverse)
            || (replay_info.ix == 0 && initial.reverse)
        {
            replay_info.playing = None;
        }
    }
}

fn adjust_replay(mut replay_info: ResMut<ReplayInfo>, input: Res<Input<KeyCode>>, time: Res<Time>) {
    let record_frame = replay_info.frame;
    let real_frame = discretized_time(&time);

    if input.just_pressed(KeyCode::Space) {
        if replay_info.playing.is_some() {
            replay_info.playing = None;
        } else {
            replay_info.playing = Some(ActiveReplayMeta {
                record_frame,
                real_frame,
                reverse: false,
            });
        }
    }

    if input.just_pressed(KeyCode::R) {
        if matches!(
            replay_info.playing,
            Some(ActiveReplayMeta { reverse: true, .. })
        ) {
            replay_info.playing = None;
        } else {
            replay_info.playing = Some(ActiveReplayMeta {
                record_frame,
                real_frame,
                reverse: true,
            })
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
            (adjust_replay, advance_frame)
                .chain()
                .run_if(in_state(MainState::PostGame)),
        )
        .add_systems(OnEnter(MainState::PostGame), initialize_replay)
        .add_systems(OnExit(MainState::PostGame), cleanup_replay);
    }
}
