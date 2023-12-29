use bevy::{
    ecs::{
        change_detection::DetectChanges,
        system::{Query, Res, ResMut, Resource},
        world::Ref,
    },
    time::Time,
};

use super::{queue::PieceQueue, Active, Hold, Matrix, MatrixUpdate, Mino, MinoKind};

#[derive(Resource, Default, Debug)]
pub struct Record {
    pub data: Vec<RecordItem>,
}

#[derive(Debug)]
pub struct RecordItem {
    pub time: u64,
    pub data: Update,
}

#[derive(Debug)]
pub enum Update {
    ActiveChange { new_position: Option<Mino> },
    QueueChange { new_queue: Box<[MinoKind]> },
    Hold { replace_with: Hold },
    MatrixChange { update: MatrixUpdate },
}

#[derive(Resource)]
pub struct FirstFrame(pub u64);

/// Discretizes time into 60ths of a second
pub fn discretized_time(time: &Time) -> u64 {
    (time.elapsed().as_millis() * 60 / 1000) as u64
}

pub(super) fn record(
    state: Query<(Ref<Active>, Ref<PieceQueue>, Ref<Hold>, Ref<Matrix>)>,
    mut record: ResMut<Record>,
    time: Res<Time>,
    first_frame: Res<FirstFrame>,
) {
    let current_frame = discretized_time(&time);
    let dt = current_frame - first_frame.0;
    for (active, queue, hold, matrix) in state.iter() {
        if active.is_changed() || active.is_added() {
            record.data.push(RecordItem {
                data: Update::ActiveChange {
                    new_position: active.0,
                },
                time: dt,
            })
        }

        if queue.is_changed() || queue.is_added() {
            let q = Vec::from(queue.window().clone()).into_boxed_slice();
            record.data.push(RecordItem {
                data: Update::QueueChange { new_queue: q },
                time: dt,
            })
        }

        if hold.is_changed() || hold.is_added() {
            record.data.push(RecordItem {
                data: Update::Hold {
                    replace_with: *hold,
                },
                time: dt,
            })
        }

        if matrix.is_changed() || matrix.is_added() {
            for &up in &matrix.updates {
                record.data.push(RecordItem {
                    data: Update::MatrixChange { update: up },
                    time: dt,
                })
            }
        }
    }
}
