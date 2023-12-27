use bevy::prelude::*;

use crate::{
    assets::MinoTextures,
    board::{Bounds, Matrix, MatrixSprite, CELL_SIZE},
    image_tools::copy_from_to,
};

use super::AddedOrChanged;

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
pub(super) fn redraw_board(
    mut board: Query<(&mut Matrix, &Children), AddedOrChanged<Matrix>>,
    children: Query<&Handle<Image>, With<MatrixSprite>>,
    mut texture_server: ResMut<Assets<Image>>,
    mino_textures: Res<MinoTextures>,
) {
    for (mut board, ch) in board.iter_mut() {
        let texture_id = ch.iter().find_map(|c| children.get(*c).ok()).unwrap();

        let mut image = texture_server.get(texture_id).cloned().unwrap();

        for up in board.updates.drain(..) {
            let tex = up.kind.select(&mino_textures);
            let replace_image = texture_server.get(tex).unwrap();
            copy_from_to(&mut image, replace_image, up.loc);
        }

        *texture_server.get_mut(texture_id).unwrap() = image;
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
