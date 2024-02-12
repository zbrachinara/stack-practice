use crate::board::{queue::PieceQueue, Active, BoardQueryItem, Hold, Matrix, MatrixUpdate, Mino};
use bevy::prelude::*;
use std::ops::{Index, Range};
use std::sync::Arc;

#[derive(Deref, DerefMut, Default, Debug)]
pub struct RecordSegment {
    #[deref]
    data: Vec<RecordItem>,
    children: Vec<(usize, Arc<RecordSegment>)>,
}

/// The record being built by the current game
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct PartialRecord(RecordSegment);

/// The chain of segments that the player is currently viewing
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct CompleteRecord {
    #[deref]
    segments: Vec<Arc<RecordSegment>>,
    separations: Vec<usize>,
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

impl CompleteRecord {
    pub fn last_frame(&self) -> u64 {
        self.last().unwrap().last().unwrap().time
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        *self.separations.last().unwrap() + self.segments.last().unwrap().data.len()
    }

    pub fn get(&self, range: Range<usize>) -> RecordSlice {
        RecordSlice {
            record: self,
            range,
        }
    }

    pub fn add_segment(&mut self, segment: RecordSegment) {
        if let Some(parent) = self.segments.last() {
            unimplemented!("Insert child into parent")
        } else {
            self.separations = vec![0];
            self.segments = vec![Arc::new(segment)];
        }
    }
}

impl Index<usize> for CompleteRecord {
    type Output = RecordItem;

    fn index(&self, index: usize) -> &Self::Output {
        let (segment_no, segment_pt) = self
            .separations
            .iter()
            .enumerate()
            .find(|(_, sep)| **sep <= index)
            .unwrap();
        &self.segments[segment_no][index - segment_pt]
    }
}

pub struct RecordSlice<'a> {
    record: &'a CompleteRecord,
    range: Range<usize>,
}

impl<'a> RecordSlice<'a> {
    pub fn iter(&self) -> RecordSliceIter {
        RecordSliceIter {
            position: self.range.start,
            rposition: self.range.end,
            slice: self.record,
        }
    }
}

pub struct RecordSliceIter<'a> {
    slice: &'a CompleteRecord,
    position: usize,
    rposition: usize,
}

impl<'a> Iterator for RecordSliceIter<'a> {
    type Item = &'a RecordItem;

    fn next(&mut self) -> Option<Self::Item> {
        (self.position < self.rposition).then(|| {
            let item = &self.slice[self.position]; // TODO maybe save which segment we are on as optimization
            self.position += 1;
            item
        })
    }
}

impl<'a> DoubleEndedIterator for RecordSliceIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.position < self.rposition).then(|| {
            self.rposition -= 1;
            &self.slice[self.rposition]
        })
    }
}

#[derive(Resource)]
pub struct FirstFrame(pub u64);

/// Discretizes time into 60ths of a second
pub fn discretized_time(time: &Time) -> u64 {
    (time.elapsed().as_millis() * 60 / 1000) as u64
}

pub(crate) fn record(
    state: Query<(Ref<Active>, Ref<PieceQueue>, Ref<Hold>, Ref<Matrix>)>,
    mut record: ResMut<PartialRecord>,
    time: Res<Time>,
    first_frame: Res<FirstFrame>,
) {
    let current_frame = discretized_time(&time);
    let dt = current_frame - first_frame.0;
    for (active, queue, hold, matrix) in state.iter() {
        if active.is_changed() {
            record.push(RecordItem {
                data: RecordData::ActiveChange {
                    new_position: active.0,
                },
                time: dt,
            })
        }

        if queue.is_changed() {
            record.push(RecordItem {
                data: RecordData::QueueChange {
                    new_queue: queue.clone(),
                },
                time: dt,
            })
        }

        if hold.is_changed() {
            record.push(RecordItem {
                data: RecordData::Hold {
                    replace_with: *hold,
                },
                time: dt,
            })
        }

        if matrix.is_changed() {
            for &up in &matrix.updates {
                record.push(RecordItem {
                    data: RecordData::MatrixChange { update: up },
                    time: dt,
                })
            }
        }
    }
}

pub(crate) fn finalize_record(
    mut complete: ResMut<CompleteRecord>,
    mut finished: ResMut<PartialRecord>,
) {
    complete.add_segment(std::mem::take(&mut **finished));
}

impl<'world> BoardQueryItem<'world> {
    pub fn apply_record(&mut self, record: &RecordItem) {
        match &record.data {
            RecordData::ActiveChange { new_position } => self.active.0 = *new_position,
            RecordData::QueueChange { new_queue } => *(self.queue) = new_queue.clone(),
            RecordData::Hold { replace_with } => *(self.hold) = *replace_with,
            RecordData::MatrixChange { update } => {
                self.matrix.updates.push(*update);
                self.matrix.data[update.loc.y as usize][update.loc.x as usize] = update.new;
            }
        }
    }

    /// This function undoes a record which has been previously been applied through
    /// [`Self::apply_record`]. This can be used, for example, to rewind through a record.
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
