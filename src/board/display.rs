use bevy::{
    asset::{Assets, Handle},
    ecs::{
        query::{Added, Changed, Or, With},
        system::{Query, Res, ResMut},
    },
    hierarchy::Children,
    render::{color::Color, texture::Image, view::Visibility},
    sprite::Sprite,
    transform::components::Transform,
};
use itertools::Itertools;

use crate::assets::{
    tables::{shape_table::ShapeParameters, sprite_table::SpriteTable},
    MinoTextures,
};

use super::{
    copy_from_to, queue::PieceQueue, Active, ActiveSprite, BoardTextures, Bounds, Hold, HoldSprite,
    Matrix, MatrixSprite, QueueSprite, RotationState, CELL_SIZE,
};

type AddedOrChanged<T> = Or<(Added<T>, Changed<T>)>;

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
pub(super) fn redraw_board(
    mut board: Query<(&BoardTextures, &mut Matrix), AddedOrChanged<Matrix>>,
    mut texture_server: ResMut<Assets<Image>>,
    mino_textures: Res<MinoTextures>,
) {
    for (textures, mut board) in board.iter_mut() {
        let mut image = texture_server
            .get(textures.matrix_cells.clone())
            .cloned()
            .unwrap();

        for up in board.updates.drain(..) {
            let tex = up.kind.select(&mino_textures);
            let replace_image = texture_server.get(tex).unwrap();
            copy_from_to(&mut image, replace_image, up.loc);
        }

        *texture_server
            .get_mut(textures.matrix_cells.clone())
            .unwrap() = image;
    }
}

pub(super) fn center_board(
    boards: Query<(&Bounds, &Children), AddedOrChanged<Bounds>>,
    mut sprites: Query<&mut Transform, With<MatrixSprite>>,
) {
    for (board, children) in boards.iter() {
        let board_bounds = board.true_bounds.as_vec2();
        let legal_bounds = board.legal_bounds.as_vec2();
        let offset = (board_bounds / 2. - legal_bounds / 2.) * (CELL_SIZE as f32);

        let child = *children.iter().find(|q| sprites.contains(**q)).unwrap();
        sprites.get_mut(child).unwrap().translation = offset.extend(0.0);
    }
}

/// Updates the visual state of the active piece. The active piece is a child of the board,
/// initialized in the same system that spawns the board. If the active pice becomes `None`, then
/// the sprite representing it is hidden. If it is modified in any other way, the sprite's position
/// and kind will be updated to match.
pub(super) fn display_active(
    active: Query<(&Active, &Bounds, &Children), AddedOrChanged<Active>>,
    mut sprites: Query<(&mut Visibility, &mut Transform, &mut Handle<Image>), With<ActiveSprite>>,
    sprite_table: Res<SpriteTable>,
) {
    for (Active(e), bounds, children) in active.iter() {
        let active_sprite_id = children.iter().copied().find(|&c| sprites.contains(c));
        let (mut vis, mut pos, mut tex) = sprites.get_mut(active_sprite_id.unwrap()).unwrap();

        if let Some(piece) = e {
            *vis = Visibility::Inherited;

            let offset = -(bounds.legal_bounds.as_vec2() / 2.);
            let new_pos = (piece.position.as_vec2() + offset) * CELL_SIZE as f32;
            pos.translation = new_pos.extend(1.0);

            *tex = sprite_table.0[&ShapeParameters {
                kind: piece.kind,
                rotation: piece.rotation,
            }]
                .clone();
        } else {
            *vis = Visibility::Hidden
        }
    }
}

// TODO: This function does not react to changes to queue window size
// TODO: This function does not react to changes in matrix bounds
/// Updates the visual state of the piece queue. When the queue changes, each piece in the queue has
/// its texture updated to match its intended state.
pub(super) fn display_queue(
    queue: Query<(&PieceQueue, &Children), AddedOrChanged<PieceQueue>>,
    mut sprites: Query<(&mut Handle<Image>, &QueueSprite)>,
    sprite_table: Res<SpriteTable>,
) {
    for (queue, children) in queue.iter() {
        for e in children
            .iter()
            .copied()
            .filter(|&e| sprites.contains(e))
            .collect_vec()
        {
            let (mut tex, QueueSprite(n)) = sprites.get_mut(e).unwrap();
            let selector = ShapeParameters {
                kind: queue.window()[*n],
                rotation: RotationState::Up,
            };
            *tex = sprite_table.0[&selector].clone();
        }
    }
}

/// Displays the held piece. Greys the texture of the associated sprite if it is inactive, or keeps
/// it at its normal color if it is not. The sprite is hidden if the hold slot is empty.
pub(super) fn display_held(
    hold: Query<(&Hold, &Children), AddedOrChanged<Hold>>,
    mut sprites: Query<(&mut Visibility, &mut Sprite, &mut Handle<Image>), With<HoldSprite>>,
    sprite_table: Res<SpriteTable>,
) {
    for (hold, children) in hold.iter() {
        let child = children
            .iter()
            .copied()
            .find(|&c| sprites.contains(c))
            .unwrap();
        let (mut vis, mut spr, mut tex) = sprites.get_mut(child).unwrap();

        if let &Hold::Active(p) | &Hold::Inactive(p) = hold {
            let selector = ShapeParameters {
                kind: p,
                rotation: RotationState::Up,
            };
            *tex = sprite_table.0[&selector].clone();
        }

        match hold {
            Hold::Empty => {
                *vis = Visibility::Hidden;
            }
            Hold::Inactive(_) => {
                *vis = Visibility::Inherited;
                let greying = 0.3;
                spr.color = Color::Rgba {
                    red: greying,
                    green: greying,
                    blue: greying,
                    alpha: 0.8,
                };
            }
            Hold::Active(_) => {
                *vis = Visibility::Inherited;
                spr.color = Color::WHITE;
            }
        }
    }
}
