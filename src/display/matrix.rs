use crate::assets::matrix_material::{MatrixMaterial, MatrixMaterialSpawner};
use bevy::prelude::*;

use crate::board::{Bounds, Matrix, CELL_SIZE, MATRIX_DEFAULT_SIZE};

#[derive(Component)]
pub struct MatrixSprite;

pub(crate) fn spawn_matrix_sprite(
    mut commands: Commands,
    boards: Query<Entity, Added<Matrix>>,
    mut mesh_spawner: MatrixMaterialSpawner,
) {
    for e in boards.iter() {
        let matrix_sprite = mesh_spawner
            .spawn_centered(MATRIX_DEFAULT_SIZE)
            .insert(MatrixSprite)
            .id();

        commands.entity(e).add_child(matrix_sprite);
    }
}

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
pub(crate) fn redraw_board(
    board: Query<(&Matrix, &Bounds, &Children), Changed<Matrix>>,
    children: Query<&Handle<MatrixMaterial>, With<MatrixSprite>>,
    mut material_server: ResMut<Assets<MatrixMaterial>>,
) {
    for (board, bounds, ch) in board.iter() {
        let material_id = ch.iter().find_map(|c| children.get(*c).ok()).unwrap();
        let material = material_server.get_mut(material_id).unwrap();

        let cells = board.data.iter().enumerate().flat_map(|(y, r)| {
            r.iter()
                .enumerate()
                .map(move |(x, c)| (y * bounds.true_bounds.x as usize + x, c))
        });

        for (ix, data) in cells {
            material.data[ix] = *data as u32;
        }
    }
}

/// Centers the legal part of the matrix rather than the entire matrix.
pub(crate) fn center_board(
    boards: Query<(&Bounds, &Children), Changed<Bounds>>,
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
