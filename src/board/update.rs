use std::ops::Deref;

use bevy::math::ivec2;
use bevy::prelude::*;
use tap::{Tap, TapOptional};

use crate::assets::tables::{
    kick_table::{KickParameters, KickTable},
    shape_table::ShapeTable,
    QueryKickTable, QueryShapeTable,
};
use crate::controller::{Controller, RotateCommand};
use crate::state::MainState;

use super::{BoardQuery, BoardQueryItem, Hold, Matrix, Mino, MinoKind, RotationState};

/// Checks if the matrix can accommodate the given piece.
fn has_free_space(matrix: &Matrix, mino: Mino, shape_table: &ShapeTable) -> bool {
    shape_table[mino]
        .iter()
        .map(|&shape_offset| shape_offset + mino.position)
        .all(|position| matrix.get(position) == Some(MinoKind::E))
}

/// Lock the given piece into the matrix, at the position and rotation it comes with. If there were
/// any filled cells that take up the same space as the given mino, those cells are overwritten with
/// the new piece. Line clears are also applied to the matrix, and any updates to the texture of the
/// matrix are also registered.
fn lock_piece(matrix: &mut Matrix, mino: Mino, shape_table: &ShapeTable) {
    for &p in &shape_table[mino] {
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
    fn maximum_valid<F>(&self, table: &ShapeTable, mut f: F) -> Option<i32>
    where
        F: FnMut(i32) -> Mino,
    {
        (0..)
            .find(|o| !has_free_space(&self.matrix, f(*o), table))
            .and_then(|o| (o > 0).then_some(o - 1))
    }

    fn drop_height(&mut self, shape_table: &ShapeTable, active: Mino) -> i32 {
        self.maximum_valid(shape_table, |y| active.tap_mut(|p| p.position.y -= y))
            .unwrap()
    }

    /// If the controller requests that the active piece is shifted, the piece will be shifted and
    /// marked as modified. Returns true if the shift was successful.
    fn shift(&mut self, controller: &Controller, shape_table: &ShapeTable) -> bool {
        let farthest_shift_left = -self
            .maximum_valid(shape_table, |x| {
                self.active().tap_mut(|p| p.position.x -= x)
            })
            .unwrap();
        let farthest_shift_right = self
            .maximum_valid(shape_table, |x| {
                self.active().tap_mut(|p| p.position.x += x)
            })
            .unwrap();

        let shift_size = controller
            .shift
            .clamp(farthest_shift_left, farthest_shift_right);

        (shift_size != 0).tap(|&shifting| {
            if shifting {
                self.active_mut().position.x += shift_size;
            }
        })
    }

    fn rotate(
        &mut self,
        controller: &Controller,
        kick_table: &KickTable,
        shape_table: &ShapeTable,
    ) -> bool {
        let original_rotation = self.active().rotation;
        let Some(new_rotation) = controller.rotation.map(|command| match command {
            RotateCommand::Left => original_rotation.rotate_left(),
            RotateCommand::Right => original_rotation.rotate_right(),
            RotateCommand::R180 => original_rotation.rotate_180(),
        }) else {
            return false;
        };

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

        successful_rot
            .tap_some(|&rot| {
                *self.active_mut() = rot;
            })
            .is_some()
    }

    fn hard_drop(&mut self, shape_table: &ShapeTable, state: &mut NextState<MainState>) {
        let mut active = self.take_active();
        active.position.y -= self.drop_height(shape_table, active);
        lock_piece(&mut self.matrix, active, shape_table);
        let new_piece = self.queue.peek();
        if !self.spawn_piece(default_mino(new_piece), shape_table) {
            state.0 = Some(MainState::PostGame);
        } else {
            self.queue.take();
            self.hold.activate();
        }
    }

    /// Switches the held piece and the active piece, if it is allowed. By this point, the active
    /// piece must exist.
    fn switch_hold_active(&mut self) -> Option<MinoKind> {
        match self.hold.deref() {
            Hold::Empty => {
                *(self.hold) = Hold::Inactive(self.take_active().kind);
                Some(self.queue.take())
            }
            Hold::Ready(piece) => {
                let piece = *piece;
                *(self.hold) = Hold::Inactive(self.take_active().kind);
                Some(piece)
            }
            Hold::Inactive(_) => None,
        }
    }

    /// Reset the board to its original state (matrix, hold, queue)
    pub fn clear_board(&mut self) {
        *(self.hold) = Hold::Empty;
        self.active.0 = None;
        *self.queue = default(); // TODO empty the queue instead of filling it with arbitrary data
    }

    /// Attempts to spawn the given piece on the board, returning whether spawning was successful.
    pub fn spawn_piece(&mut self, piece: Mino, shape_table: &ShapeTable) -> bool {
        has_free_space(&self.matrix, piece, shape_table).tap(|&has_free_space| {
            if has_free_space {
                *self.drop_clock = default();
                self.active.0 = Some(piece);
            }
        })
    }
}

// TODO this should be determined at runtime
pub fn default_mino(kind: MinoKind) -> Mino {
    Mino {
        kind,
        position: ivec2(4, 22),
        rotation: RotationState::Up,
    }
}

/// Update the state of the memory-representation of the board using player input
pub(crate) fn update_board(
    mut boards: Query<BoardQuery>,
    controller: Res<Controller>,
    shape_table: QueryShapeTable,
    kick_table: QueryKickTable,
    time: Res<Time>,
    mut state: ResMut<NextState<MainState>>,
) {
    for mut board in boards.iter_mut() {
        if board.active.deref().0.is_none() {
            continue;
        }

        if controller.hard_drop {
            board.hard_drop(&shape_table, &mut state);
            continue;
        }

        let farthest_legal_drop = board.drop_height(&shape_table, board.active());

        // The drop clock should only either drop the piece or lock it, NOT BOTH. This is so
        // that the player has time to interact with the piece when it hits the bottom, for a
        // frame at the very least. Later, we may want to rethink this for zero lock delay, if
        // such a thing makes sense.
        if farthest_legal_drop == 0 {
            board.drop_clock.lock += time.delta_seconds();
            if board.drop_clock.lock > board.settings.lock_delay {
                board.hard_drop(&shape_table, &mut state);
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

        let rotation_success = board.rotate(&controller, &kick_table, &shape_table);
        let shift_success = board.shift(&controller, &shape_table);

        if rotation_success || shift_success {
            // TODO also modify a lock reset counter
            board.drop_clock.lock = 0.0;
        }

        if controller.hold {
            if let Some(replace) = board.switch_hold_active() {
                if !board.spawn_piece(default_mino(replace), &shape_table) {
                    state.0 = Some(MainState::PostGame);
                }
            }
        }
    }
}
