use std::ops::Deref;

use bevy::{
    ecs::{
        query::WorldQuery,
        system::{Query, Res},
    },
    math::ivec2,
    utils::default,
};
use tap::Tap;

use crate::assets::tables::{
    kick_table::{KickParameters, KickTable},
    shape_table::{ShapeParameters, ShapeTable},
    QueryKickTable, QueryShapeTable,
};

use super::{
    controller::Controller, queue::PieceQueue, Active, Bounds, DropClock, Hold, Matrix,
    MatrixUpdate, Mino, MinoKind, RotationState, TEXTURE_CENTER_OFFSET,
};

/// Checks if the matrix can accomodate the given piece.
fn has_free_space(matrix: &Matrix, mino: Mino, shape_table: &ShapeTable) -> bool {
    shape_table.0[&ShapeParameters::from(&mino)]
        .iter()
        .map(|&shape_offset| shape_offset + mino.position)
        .all(|position| matrix.get(position) == Some(MinoKind::E))
}

/// Lock the given piece into the matrix, at the position and rotation it comes with.
fn lock_piece_at(matrix: &mut Matrix, mino: Mino, shape_table: &ShapeTable) {
    let old_board = matrix.data.clone();

    for &p in &shape_table.0[&ShapeParameters::from(&mino)] {
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

    // TODO documentation, make this function more transparent
    fn maximum_for_which<F>(&self, table: &ShapeTable, mut f: F) -> i32
    where
        F: FnMut(i32) -> Mino,
    {
        (1..)
            .map(|o| (o, f(o)))
            .find(|(_, mino)| !has_free_space(&self.matrix, *mino, table))
            .map(|(o, _)| o - 1)
            .unwrap()
    }

    /// If the controller requests that the active piece is shifted, the piece will be shifted and
    /// marked as modified.
    fn do_shift(&mut self, controller: &Controller, shape_table: &ShapeTable) {
        let farthest_shift_left = self.maximum_for_which(shape_table, |x| {
            self.active().tap_mut(|p| p.position.x -= x)
        });

        let farthest_shift_right = self.maximum_for_which(shape_table, |x| {
            self.active().tap_mut(|p| p.position.x += x)
        });

        let shift_size = if controller.shift_left {
            -std::cmp::min(1, farthest_shift_left)
        } else if controller.shift_right {
            std::cmp::min(1, farthest_shift_right)
        } else if controller.repeat_left {
            -std::cmp::min(SHIFT_SIZE, farthest_shift_left)
        } else if controller.repeat_right {
            std::cmp::min(SHIFT_SIZE, farthest_shift_right)
        } else {
            0
        };

        if shift_size != 0 {
            self.active_mut().position.x += shift_size;
        }
    }

    fn do_rotate(
        &mut self,
        controller: &Controller,
        kick_table: &KickTable,
        shape_table: &ShapeTable,
    ) {
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
            }
        }
    }

    /// Switches the held piece and the active piece, if it is allowed. By this point, the active
    /// piece must exist.
    fn switch_hold_active(&mut self) {
        let replace_active = if let &Hold::Active(p) = self.hold.deref() {
            *(self.hold) = Hold::Inactive(self.active().kind);
            Some(p)
        } else if matches!(self.hold.deref(), Hold::Empty) {
            *(self.hold) = Hold::Inactive(self.active().kind);
            Some(self.queue.take())
        } else {
            None
        };

        if let Some(kind) = replace_active {
            self.active.0 = Some(Mino {
                kind,
                position: ivec2(4, 22) - TEXTURE_CENTER_OFFSET,
                rotation: RotationState::Up,
            })
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
}

const SOFT_DROP_POWER: f32 = 10.0;
const SHIFT_SIZE: i32 = 1;
const GRAVITY_POWER: f32 = 0.02;
const LOCK_DELAY: f32 = 0.5;

/// Update the state of the memory-representation of the board using player input
pub(super) fn update_board(
    mut boards: Query<BoardQuery>,
    controller: Res<Controller>,
    shape_table: QueryShapeTable,
    kick_table: QueryKickTable,
) {
    for mut board in boards.iter_mut() {
        if let Some(mut p) = board.active.deref().0 {
            let farthest_legal_drop =
                board.maximum_for_which(&shape_table, |y| p.tap_mut(|p| p.position.y -= y));

            if controller.hard_drop {
                // TODO when passive effects are added, this needs to happen when the piece locks
                // (by gravity or otherwise), not just during hard drop
                board.active.0.take();
                *board.drop_clock = default();
                board.hold.activate();

                p.position.y -= farthest_legal_drop;
                lock_piece_at(&mut board.matrix, p, &shape_table);
            }
        }

        if board.active.deref().0.is_none() {
            // TODO confirm that the piece can spawn before spawning it
            board.active.0 = Some(Mino {
                kind: board.queue.take(),
                position: ivec2(4, 22) - TEXTURE_CENTER_OFFSET,
                rotation: RotationState::Up,
            });
        }

        let farthest_legal_drop = board.maximum_for_which(&shape_table, |y| {
            board.active().tap_mut(|p| p.position.y -= y)
        });

        // The drop clock should only either drop the piece or lock it, NOT BOTH. This is so
        // that the player has time to interact with the piece when it hits the bottom, for a
        // frame at the very least. Later, we may want to rethink this for zero lock delay, if
        // such a thing makes sense.
        if farthest_legal_drop == 0 {
            board.drop_clock.lock += 1. / 60.;
            if board.drop_clock.lock > LOCK_DELAY {
                // TODO perform lock
            }
        } else {
            board.drop_clock.fall += if controller.soft_drop {
                SOFT_DROP_POWER * GRAVITY_POWER
            } else {
                GRAVITY_POWER
            };
            let old_drop_clock = board.drop_clock.deref().fall;
            if old_drop_clock > 1.0 {
                board.drop_clock.fall = old_drop_clock.fract();
                let drop_distance =
                    std::cmp::min(old_drop_clock.trunc() as i32, farthest_legal_drop);
                board.active_mut().position.y -= drop_distance;
            }
        }

        board.do_rotate(&controller, &kick_table, &shape_table);
        board.do_shift(&controller, &shape_table);
        if controller.hold {
            board.switch_hold_active();
        }
    }
}
