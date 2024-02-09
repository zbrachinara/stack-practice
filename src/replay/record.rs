use bevy::{
    ecs::{
        change_detection::DetectChanges,
        system::{Query, Res, ResMut, Resource},
        world::Ref,
    },
    time::Time,
};

use crate::board::{
    queue::PieceQueue, Active, BoardQueryItem, Hold, Matrix, MatrixAction, MatrixUpdate, Mino,
    MinoKind,
};

#[derive(Resource, Default, Debug)]
pub struct Record {
    pub data: Vec<RecordItem>,
}

#[derive(Debug)]
pub struct RecordItem {
    pub time: u64,
    pub data: RecordData,
}

#[derive(Debug)]
pub enum RecordData {
    ActiveChange { new_position: Option<Mino> },
    QueueChange { new_queue: PieceQueue },
    Hold { replace_with: Hold },
    MatrixChange { update: MatrixUpdate },
}

#[derive(Resource)]
pub struct FirstFrame(pub u64);

/// Discretizes time into 60ths of a second
pub fn discretized_time(time: &Time) -> u64 {
    (time.elapsed().as_millis() * 60 / 1000) as u64
}

pub(crate) fn record(
    state: Query<(Ref<Active>, Ref<PieceQueue>, Ref<Hold>, Ref<Matrix>)>,
    mut record: ResMut<Record>,
    time: Res<Time>,
    first_frame: Res<FirstFrame>,
) {
    let current_frame = discretized_time(&time);
    let dt = current_frame - first_frame.0;
    for (active, queue, hold, matrix) in state.iter() {
        if active.is_changed() {
            record.data.push(RecordItem {
                data: RecordData::ActiveChange {
                    new_position: active.0,
                },
                time: dt,
            })
        }

        if queue.is_changed() {
            record.data.push(RecordItem {
                data: RecordData::QueueChange {
                    new_queue: queue.clone(),
                },
                time: dt,
            })
        }

        if hold.is_changed() {
            record.data.push(RecordItem {
                data: RecordData::Hold {
                    replace_with: *hold,
                },
                time: dt,
            })
        }

        if matrix.is_changed() {
            for &up in &matrix.updates {
                record.data.push(RecordItem {
                    data: RecordData::MatrixChange { update: up },
                    time: dt,
                })
            }
        }
    }
}

impl<'world> BoardQueryItem<'world> {
    pub fn apply_record(&mut self, record: &RecordItem) {
        match &record.data {
            RecordData::ActiveChange { new_position } => self.active.0 = *new_position,
            RecordData::QueueChange { new_queue } => *(self.queue) = new_queue.clone(),
            RecordData::Hold { replace_with } => *(self.hold) = *replace_with,
            RecordData::MatrixChange { update } => {
                self.matrix.updates.push(*update);

                self.matrix.data[update.loc.y as usize][update.loc.x as usize] =
                    if update.action == MatrixAction::Insert {
                        update.kind
                    } else {
                        MinoKind::E
                    };
            }
        }
    }

    /// This function undoes a record which has been previously been applied through [`Self::apply_record`]. This can
    /// be used, for example, to rewind through a record.
    pub fn undo_record(&mut self, record: &RecordItem) {
        match &record.data {
            RecordData::MatrixChange { update } => {
                let update = update.invert();
                self.apply_record(&RecordItem {
                    data: RecordData::MatrixChange { update },
                    time: record.time,
                }) // TODO this should be cleaner (no need to duplicate time, etc)
            }
            _ => self.apply_record(record),
        }
    }
}
