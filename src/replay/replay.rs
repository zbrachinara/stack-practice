//! Replay code currently depends on the board being unique in the world.

use crate::animation::{CameraZoom, DEFAULT_CAMERA_ZOOM, REPLAY_CAMERA_ZOOM};
use crate::progress_bar::{ProgressBar, ProgressBarBundle, ProgressBarMaterial};
use crate::replay::record::discretized_time;
use crate::replay::record::{CompleteRecord, RecordData};
use bevy::prelude::*;
use duplicate::duplicate;
use itertools::Itertools;

use crate::board::{Active, BoardQuery};
use crate::controller::{Controller, ControllerFrozen};
use crate::state::MainState;

/// Stores information about the state of the replay (i.e. paused or played, frames progressed).
#[derive(Resource, Default, Debug)]
pub struct ReplayInfo {
    /// The current frame the replay occupies.
    pub frame: u64,
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

#[derive(Component)]
pub struct ReplayBar;

pub(crate) fn setup_progress_bar(
    mut commands: Commands,
    mut materials: ResMut<Assets<ProgressBarMaterial>>,
    record: Res<CompleteRecord>,
) {
    let style = Style {
        position_type: PositionType::Absolute,
        height: Val::Percent(95.0),
        width: Val::Px(2.0),
        right: Val::Percent(5.0),
        top: Val::Percent(2.5),
        ..default()
    };

    commands
        .spawn(ProgressBarBundle {
            progressbar: ProgressBar {
                sections: record
                    .segments
                    .iter()
                    .enumerate()
                    .map(|(ix, segment)| {
                        let time = segment.last().unwrap().time;
                        let color = Color::hsl(0., 0.5, 0.85f32.powi(ix as i32));
                        (time as u32, color)
                    })
                    .collect_vec(),
                ..default()
            },
            material_node_bundle: MaterialNodeBundle {
                material: materials.add(ProgressBarMaterial::default()),
                style,
                ..default()
            },
        })
        .insert(ReplayBar);
}

pub(crate) fn remove_progress_bar(mut commands: Commands, bar: Query<Entity, With<ReplayBar>>) {
    commands.entity(bar.single()).despawn_recursive();
}

pub(crate) fn update_progress(
    mut bar: Query<&mut ProgressBar, With<ReplayBar>>,
    info: Res<ReplayInfo>,
    record: Res<CompleteRecord>,
) {
    bar.single_mut().progress = info.frame as f32 / record.last_frame() as f32;
}

pub fn initialize_replay(
    mut commands: Commands,
    record: Res<CompleteRecord>,
    mut zoom: ResMut<CameraZoom>,
) {
    **zoom = REPLAY_CAMERA_ZOOM;

    let replay_info = ReplayInfo {
        frame: record.last_frame(),
        ix: record.len(),
        next_ix: record.len(),
        playing: None,
    };

    tracing::info!("Entering replay with {replay_info:?}");
    commands.insert_resource(replay_info);
}

pub fn cleanup_replay(mut zoom: ResMut<CameraZoom>) {
    **zoom = DEFAULT_CAMERA_ZOOM;
}

pub fn replay(
    record: Res<CompleteRecord>,
    mut replay_info: ResMut<ReplayInfo>,
    mut board: Query<BoardQuery>,
) {
    let mut board = board.single_mut();
    if let Some(meta) = replay_info.playing {
        if meta.reverse {
            // Reaching past next_ix to find the current active piece, hold, and queue. This is
            // necessary because these properties can span multiple frames past when they are
            // applied. For example, when dealing with updates to the active piece, the piece may
            // stay in the same position for multiple frames while it locks onto the floor. However,
            // the record is only on the frame when it touches the floor, not when it locks. Thus,
            // when we rewind, the board will update to show that the piece has not been placed yet,
            // but the active piece will not become visible until we get to the first frame it
            // touches the floor (this phenomenon actually applies to the active piece's position in
            // general, but this illustration is much more vivid, because it will appear that the
            // board doesn't actually have an active piece).
            let search = record.get(0..replay_info.next_ix);
            duplicate! {
                [
                    Match; [ActiveChange]; [Hold]; [QueueChange];
                ]

                if let Some(update) = search
                    .iter()
                    .rev()
                    .find(|i| matches!(i.data, RecordData::Match { .. }))
                {
                    board.apply_record(update);
                }
            }

            // matrix changes can be applied immediately
            for item in record
                .get(replay_info.next_ix..replay_info.ix)
                .iter()
                .filter(|i| matches!(i.data, RecordData::MatrixChange { .. }))
                .rev()
            {
                board.undo_record(item);
            }
        } else {
            for item in record.get(replay_info.ix..replay_info.next_ix).iter() {
                board.apply_record(item);
            }
        }
    }
    replay_info.ix = replay_info.next_ix;
}

pub fn advance_frame(
    mut replay_info: ResMut<ReplayInfo>,
    record: Res<CompleteRecord>,
    time: Res<Time>,
) {
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
                record
                    .get(0..std::cmp::min(replay_info.ix + 1, record.len()))
                    .iter()
                    .rev()
                    .position(|item| item.time < new_record_frame)
                    .map(|ix| replay_info.ix - ix + 1)
                    .unwrap_or(0)
            } else {
                record
                    .get(replay_info.ix..record.len())
                    .iter()
                    .position(|item| item.time > new_record_frame)
                    .map(|ix| replay_info.ix + ix)
                    .unwrap_or(record.len())
            };
        }

        // pause replay after reaching the end of the record
        if (replay_info.ix == record.len() && !initial.reverse)
            || (replay_info.ix == 0 && initial.reverse)
        {
            replay_info.playing = None;
        }
    }
}

pub(crate) fn adjust_replay(
    mut replay_info: ResMut<ReplayInfo>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
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

    if input.just_pressed(KeyCode::KeyR) {
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

#[derive(Event, Default)]
pub(crate) struct DeferUnfreeze;

// When the controller registers a movement, begins a new segment in the replay and puts the player
// in control of the game, starting from the current point of the replay. If instead, the grave key
// is pressed, we return to the ready state.
pub(crate) fn exit_replay(
    mut next_state: ResMut<NextState<MainState>>,
    controller: Res<Controller>,
    keys: Res<ButtonInput<KeyCode>>,
    active_piece: Query<&Active>,
    mut controller_freeze: ResMut<ControllerFrozen>,
    mut defer_unfreeze: EventWriter<DeferUnfreeze>,
) {
    // TODO resolve conflict between space bar for hard drop and pause/play replay

    let active_piece_exists = active_piece
        .get_single()
        .is_ok_and(|piece| piece.0.is_some());

    if controller.any_activation() && !controller.hard_drop && active_piece_exists {
        // we are branching the current record
        next_state.0 = Some(MainState::Playing);
        **controller_freeze = true;
        defer_unfreeze.send(default());
    } else if keys.just_pressed(KeyCode::Backquote) {
        // we are beginning a new record
        next_state.0 = Some(MainState::Ready);
    }
}

pub(crate) fn unfreeze_controller_after_exit(mut freeze_state: ResMut<ControllerFrozen>) {
    println!("unfreeze was run");
    **freeze_state = false;
}
