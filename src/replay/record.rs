use crate::board::{
    queue::PieceQueue, Active, BoardQueryItem, Hold, Matrix, MatrixUpdate, Mino, MinoKind,
};
use crate::replay::replay::ReplayInfo;
use bevy::math::ivec2;
use bevy::prelude::*;
use smart_default::SmartDefault;
use std::ops::{Index, Range};
use std::sync::{Arc, Mutex};

#[derive(Deref, DerefMut, Default, Debug)]
pub struct RecordSegment {
    #[deref]
    data: Vec<RecordItem>,
    children: Mutex<Vec<(u64, Arc<RecordSegment>)>>,
}

/// The record being built by the current game
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct PartialRecord(RecordSegment);

/// The chain of segments that the player is currently viewing
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct CompleteRecord {
    #[deref]
    pub segments: Vec<Arc<RecordSegment>>,
    pub separations: Vec<usize>,
}

#[derive(Debug)]
pub struct RecordItem {
    pub time: u64,
    pub data: RecordData,
}

#[derive(Debug)]
pub enum RecordData {
    ActiveChange(Option<Mino>),
    QueueChange(PieceQueue),
    Hold(Hold),
    MatrixChange(MatrixUpdate),
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
        let segment = Arc::new(segment);
        if let Some(parent) = self.segments.last_mut() {
            let first_frame = segment.first().unwrap().time;
            let mut children = parent.children.lock().unwrap();

            // find the insert location
            let location = children
                .iter()
                .position(|(t, _)| *t > first_frame)
                .unwrap_or(children.len());

            // find the separation location
            let separation_ix = parent
                .data
                .iter()
                .position(|e| e.time >= first_frame)
                .unwrap();

            children.insert(location, (first_frame, segment.clone()));
            drop(children);

            self.segments.push(segment);
            self.separations.push(separation_ix);
        } else {
            self.separations = vec![0];
            self.segments = vec![segment];
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
            .rev()
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

/// A record of what the contents of the matrix were in the previous frame. The frame transition is
/// managed by [`record`]
#[derive(Component, Deref, DerefMut, SmartDefault)]
pub struct PreviousMatrix {
    #[default(Matrix::default().data)]
    data: Vec<Vec<MinoKind>>,
}

/// Compares the contents of the new and old matrices, at the same time replacing the contents of
/// old with new. Since each update contains its own position information, the order in which the
/// updates are applied is important and should be kept.
#[allow(clippy::ptr_arg)]
fn diff_and_copy<'a>(
    // TODO better lifetime management
    new: &'a Vec<Vec<MinoKind>>,
    old: &'a mut Vec<Vec<MinoKind>>,
) -> impl Iterator<Item = MatrixUpdate> + 'a {
    let row_size = old[0].len();
    let old_mut = old.iter_mut().flat_map(|i| i.iter_mut());
    let new_updates = old_mut
        .zip(new.iter().flat_map(|i| i.iter().copied()))
        .zip((0..).map(move |ix| ivec2(ix % row_size as i32, ix / row_size as i32)))
        .filter(|((old, new), _)| *old != new)
        .map(|((old, new), loc)| {
            let update = MatrixUpdate {
                loc,
                old: *old,
                new,
            };
            *old = new;
            update
        });
    new_updates
}

pub(crate) fn record(
    mut state: Query<(
        Ref<Active>,
        Ref<PieceQueue>,
        Ref<Hold>,
        Ref<Matrix>,
        &mut PreviousMatrix,
    )>,
    mut record: ResMut<PartialRecord>,
    time: Res<Time>,
    first_frame: Res<FirstFrame>,
) {
    let current_frame = discretized_time(&time);
    let dt = current_frame - first_frame.0;
    for (active, queue, hold, matrix, mut previous_matrix) in state.iter_mut() {
        if active.is_changed() {
            record.push(RecordItem {
                data: RecordData::ActiveChange(active.0),
                time: dt,
            })
        }

        if queue.is_changed() {
            record.push(RecordItem {
                data: RecordData::QueueChange(queue.clone()),
                time: dt,
            })
        }

        if hold.is_changed() {
            record.push(RecordItem {
                data: RecordData::Hold(*hold),
                time: dt,
            })
        }

        if matrix.is_changed() {
            let updates = diff_and_copy(&matrix.data, &mut previous_matrix.data);
            record.extend(updates.map(|up| RecordItem {
                data: RecordData::MatrixChange(up),
                time: dt,
            }))
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
            RecordData::ActiveChange(new_position) => self.active.0 = *new_position,
            RecordData::QueueChange(new_queue) => *(self.queue) = new_queue.clone(),
            RecordData::Hold(replace_with) => *(self.hold) = *replace_with,
            RecordData::MatrixChange(update) => {
                self.matrix.data[update.loc.y as usize][update.loc.x as usize] = update.new;
            }
        }
    }

    /// This function undoes a record which has been previously been applied through
    /// [`Self::apply_record`]. This can be used, for example, to rewind through a record.
    pub fn undo_record(&mut self, record: &RecordItem) {
        match &record.data {
            RecordData::MatrixChange(update) => {
                let update = update.invert();
                self.apply_record(&RecordItem {
                    data: RecordData::MatrixChange(update),
                    time: record.time,
                }) // TODO this should be cleaner (no need to duplicate time, etc)
            }
            _ => self.apply_record(record),
        }
    }
}

pub(crate) fn reset_record(mut commands: Commands) {
    commands.init_resource::<PartialRecord>();
    commands.init_resource::<CompleteRecord>();
}

/// When a new record has been instantiated and a game begins, insert the [`FirstFrame`] resource
/// referring to the current frame
pub(crate) fn initialize_time(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(FirstFrame(discretized_time(&time)));
}

/// Prunes the record and cuts off and sets the first frame according to the current place
pub(crate) fn begin_new_segment(
    mut commands: Commands,
    time: Res<Time>,
    mut record: ResMut<CompleteRecord>,
    meta: Res<ReplayInfo>,
    mut boards: Query<(&Matrix, &mut PreviousMatrix)>,
) {
    commands.init_resource::<PartialRecord>();

    let offset = meta.frame;
    commands.insert_resource(FirstFrame(discretized_time(&time) - offset));

    if let Some(p) = record
        .segments
        .iter()
        .position(|seg| seg.first().unwrap().time > meta.frame)
    {
        record.segments.drain(p..);
        record.separations.drain(p..);
    }

    // Since recording does not take place during the replay, the previous frame's matrix is not
    // correct. Branching starts on the frame after the current frame of the replay, so the
    // "previous frame"'s matrix (which is in use once recording starts) should actually be the same
    // as this frame's matrix
    for (this_board, mut prev_board) in boards.iter_mut() {
        prev_board.data = this_board.data.clone()
    }
}
