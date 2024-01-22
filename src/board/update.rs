use std::ops::Deref;

use bevy::prelude::*;
use bevy::{ecs::query::WorldQuery, math::ivec2};
use tap::Tap;

use crate::assets::tables::{
    kick_table::{KickParameters, KickTable},
    shape_table::{ShapeParameters, ShapeTable},
    QueryKickTable, QueryShapeTable,
};
use crate::board::record::Update;
use crate::state::MainState;

use super::record::RecordItem;
use super::{
    controller::Controller, queue::PieceQueue, Active, Bounds, DropClock, Hold, Matrix,
    MatrixUpdate, Mino, MinoKind, RotationState, Settings,
};

/// Checks if the matrix can accommodate the given piece.
fn has_free_space(matrix: &Matrix, mino: Mino, shape_table: &ShapeTable) -> bool {
    shape_table.table[&ShapeParameters::from(&mino)]
        .iter()
        .map(|&shape_offset| shape_offset + mino.position)
        .all(|position| matrix.get(position) == Some(MinoKind::E))
}

/// Lock the given piece into the matrix, at the position and rotation it comes with. If there were
/// any filled cells that take up the same space as the given mino, those cells are overwritten with
/// the new piece. Line clears are also applied to the matrix, and any updates to the texture of the
/// matrix are also registered.
fn lock_piece(matrix: &mut Matrix, mino: Mino, shape_table: &ShapeTable) {
    let old_board = matrix.data.clone();

    for &p in &shape_table.table[&ShapeParameters::from(&mino)] {
        *(matrix.get_mut(p + mino.position).unwrap()) = mino.kind;
    }

    // line clears
    let mut real_ix = 0;
    for _ in 0..matrix.data.len() {
        if matrix.data[real_ix].iter().all(|&e| e != MinoKind::E) {
            matrix.data[real_ix..].rotate_left(1);
            matrix.data.last_mut().unwrap().fill(MinoKind::E);
        } else {
            real_ix += 1;
        }
    }

    // register updates made to the board
    let row_size = old_board[0].len();
    let new_updates = (0..).scan(
        (
            -1i32,
            old_board.into_iter().flat_map(|i| i.into_iter()),
            matrix.data.iter().flat_map(|i| i.iter()),
        ),
        |(offset, old, new), _| {
            itertools::diff_with(old.clone(), new.clone(), |a, b| a == *b).map(|d| match d {
                itertools::Diff::FirstMismatch(p, old_next, new_next) => {
                    *offset += p as i32 + 1;
                    *old = old_next.into_parts().1.clone();
                    let (Some(&kind), new_new) = new_next.into_parts() else {
                        unreachable!()
                    };
                    *new = new_new;
                    let loc = ivec2(*offset % row_size as i32, *offset / row_size as i32);
                    MatrixUpdate { loc, kind }
                }
                _ => unreachable!(),
            })
        },
    );
    matrix.updates.extend(new_updates);
}

/// Functions within this impl block will panic if the active piece does not exist.
impl<'world> BoardQueryItem<'world> {
    fn active(&self) -> Mino {
        self.active.0.unwrap()
    }
    fn active_mut(&mut self) -> &mut Mino {
        self.active.0.as_mut().unwrap()
    }
    fn take_active(&mut self) -> Mino {
        self.active.0.take().unwrap()
    }

    /// Starting from zero, finds the highest number for which the associated mino (given by `f`) is
    /// within the bounds of the matrix. If there is no such number, `None` is returned. Otherwise,
    /// the value will always be a non-negative number.
    fn maximum_for_which<F>(&self, table: &ShapeTable, mut f: F) -> Option<i32>
    where
        F: FnMut(i32) -> Mino,
    {
        (0..)
            .find(|o| !has_free_space(&self.matrix, f(*o), table))
            .and_then(|o| (o > 0).then_some(o - 1))
    }

    /// If the controller requests that the active piece is shifted, the piece will be shifted and
    /// marked as modified. Returns true if the shift was successful.
    fn do_shift(&mut self, controller: &Controller, shape_table: &ShapeTable) -> bool {
        let farthest_shift_left = self
            .maximum_for_which(shape_table, |x| {
                self.active().tap_mut(|p| p.position.x -= x)
            })
            .unwrap();
        let farthest_shift_right = self
            .maximum_for_which(shape_table, |x| {
                self.active().tap_mut(|p| p.position.x += x)
            })
            .unwrap();

        let shift_size = if controller.shift_left != 0 {
            -std::cmp::min(controller.shift_left as i32, farthest_shift_left)
        } else if controller.shift_right != 0 {
            std::cmp::min(controller.shift_right as i32, farthest_shift_right)
        } else {
            0
        };

        if shift_size != 0 {
            self.active_mut().position.x += shift_size;
            true
        } else {
            false
        }
    }

    fn do_rotate(
        &mut self,
        controller: &Controller,
        kick_table: &KickTable,
        shape_table: &ShapeTable,
    ) -> bool {
        let original_rotation = self.active().rotation;
        let rotation = if controller.rotate_180 {
            Some(original_rotation.rotate_180())
        } else if controller.rotate_left {
            Some(original_rotation.rotate_left())
        } else if controller.rotate_right {
            Some(original_rotation.rotate_right())
        } else {
            None
        };

        if let Some(new_rotation) = rotation {
            let kick_params = KickParameters {
                kind: self.active().kind,
                from: original_rotation,
                to: new_rotation,
            };
            let kicks = kick_table.0.get(&kick_params);
            let offsets =
                std::iter::once(ivec2(0, 0)).chain(kicks.iter().flat_map(|p| p.iter()).copied());

            let successful_rot = offsets
                .map(|o| {
                    self.active().tap_mut(|m| {
                        m.rotation = new_rotation;
                        m.position += o;
                    })
                })
                .find(|m| has_free_space(self.matrix.deref(), *m, shape_table));

            if let Some(successful_rot) = successful_rot {
                *self.active_mut() = successful_rot;
                return true;
            }
        }

        false
    }

    /// Switches the held piece and the active piece, if it is allowed. By this point, the active
    /// piece must exist.
    fn switch_hold_active(&mut self) -> Option<MinoKind> {
        if let &Hold::Ready(p) = self.hold.deref() {
            *(self.hold) = Hold::Inactive(self.take_active().kind);
            Some(p)
        } else if matches!(self.hold.deref(), Hold::Empty) {
            *(self.hold) = Hold::Inactive(self.take_active().kind);
            Some(self.queue.take())
        } else {
            None
        }
    }

    /// Reset the board to its original state (matrix, hold, queue)
    pub fn clear_board(&mut self) {
        *(self.hold) = Hold::Empty;
        self.active.0 = None;

        let mut updates = Vec::new();
        for (y, row) in self.matrix.data.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let p = ivec2(x as i32, y as i32);
                if *cell != MinoKind::E {
                    *cell = MinoKind::E;
                    updates.push(MatrixUpdate {
                        loc: p,
                        kind: MinoKind::E,
                    });
                }
            }
        }
        self.matrix.updates.extend(updates);

        *self.queue = default(); // TODO empty the queue instead of filling it with arbitrary data
    }

    pub fn apply_record(&mut self, record: &RecordItem) {
        match &record.data {
            Update::ActiveChange { new_position } => self.active.0 = *new_position,
            Update::QueueChange { new_queue } => *(self.queue) = new_queue.clone(),
            Update::Hold { replace_with } => *(self.hold) = *replace_with,
            Update::MatrixChange { update } => {
                self.matrix.updates.push(*update);
                self.matrix.data[update.loc.y as usize][update.loc.x as usize] = update.kind;
            }
        }
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub(super) struct BoardQuery {
    matrix: &'static mut Matrix,
    active: &'static mut Active,
    hold: &'static mut Hold,
    queue: &'static mut PieceQueue,
    drop_clock: &'static mut DropClock,
    bounds: &'static Bounds,
    settings: &'static Settings,
    id: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct PieceSpawnEvent {
    pub board: Entity,
    pub(super) mino: Mino,
}

pub(super) fn spawn_piece(
    mut events: EventReader<PieceSpawnEvent>,
    mut boards: Query<BoardQuery>,
    mut state: ResMut<NextState<MainState>>,
    shape_table: QueryShapeTable,
) {
    for &PieceSpawnEvent { board, mino } in events.read() {
        let mut board = boards.get_mut(board).unwrap();
        if has_free_space(&board.matrix, mino, &shape_table) {
            *board.drop_clock = default();
            board.active.0 = Some(mino);
        } else {
            state.0 = Some(MainState::PostGame);
        }
    }
}

/// Update the state of the memory-representation of the board using player input
pub(super) fn update_board(
    mut boards: Query<BoardQuery>,
    mut spawner: EventWriter<PieceSpawnEvent>,
    controller: Res<Controller>,
    shape_table: QueryShapeTable,
    kick_table: QueryKickTable,
    time: Res<Time>,
) {
    for mut board in boards.iter_mut() {
        if board.active.deref().0.is_none() {
            continue;
        }

        if controller.hard_drop {
            let mut active = board.take_active();
            let farthest_legal_drop = board
                .maximum_for_which(&shape_table, |y| active.tap_mut(|p| p.position.y -= y))
                .unwrap();
            active.position.y -= farthest_legal_drop;
            lock_piece(&mut board.matrix, active, &shape_table);
            spawner.send(PieceSpawnEvent {
                board: board.id,
                mino: Mino {
                    kind: board.queue.take(),
                    position: ivec2(4, 22),
                    rotation: RotationState::Up,
                },
            });
            board.hold.activate();
            continue;
        }

        let farthest_legal_drop = board
            .maximum_for_which(&shape_table, |y| {
                board.active().tap_mut(|p| p.position.y -= y)
            })
            .unwrap();

        // The drop clock should only either drop the piece or lock it, NOT BOTH. This is so
        // that the player has time to interact with the piece when it hits the bottom, for a
        // frame at the very least. Later, we may want to rethink this for zero lock delay, if
        // such a thing makes sense.
        if farthest_legal_drop == 0 {
            board.drop_clock.lock += time.delta_seconds();
            if board.drop_clock.lock > board.settings.lock_delay {
                let active = board.take_active();
                lock_piece(&mut board.matrix, active, &shape_table);
                board.hold.activate();
                spawner.send(PieceSpawnEvent {
                    board: board.id,
                    mino: Mino {
                        kind: board.queue.take(),
                        position: ivec2(4, 22),
                        rotation: RotationState::Up,
                    },
                });
                continue;
            }
        } else {
            board.drop_clock.fall += if controller.soft_drop {
                board.settings.soft_drop_power * board.settings.gravity_power
            } else {
                board.settings.gravity_power
            };
            let old_drop_clock = board.drop_clock.deref().fall;
            if old_drop_clock > 1.0 {
                board.drop_clock.fall = old_drop_clock.fract();
                let drop_distance =
                    std::cmp::min(old_drop_clock.trunc() as i32, farthest_legal_drop);
                board.active_mut().position.y -= drop_distance;
            }
        }

        let rotation_success = board.do_rotate(&controller, &kick_table, &shape_table);
        let shift_success = board.do_shift(&controller, &shape_table);

        if rotation_success || shift_success {
            // TODO also modify a lock reset counter
            board.drop_clock.lock = 0.0;
        }

        if controller.hold {
            if let Some(replace) = board.switch_hold_active() {
                spawner.send(PieceSpawnEvent {
                    board: board.id,
                    mino: Mino {
                        kind: replace,
                        position: ivec2(4, 22),
                        rotation: RotationState::Up,
                    },
                })
            }
        }
    }
}
