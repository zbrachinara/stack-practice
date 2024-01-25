use bevy::math::ivec2;
use bevy::{math::vec2, prelude::*};
use itertools::Itertools;
use tap::Tap;

use crate::assets::tables::QueryShapeTable;
use crate::assets::matrix_material::{MatrixMaterial, MatrixMaterialSpawner};
use crate::board::MinoKind;
use crate::{
    assets::tables::shape_table::ShapeParameters,
    board::{queue::PieceQueue, RotationState, CELL_SIZE, MATRIX_DEFAULT_LEGAL_BOUNDS},
};

#[derive(Component)]
pub struct QueueSprite(usize);

pub(super) fn spawn_queue_sprite(
    mut commands: Commands,
    mut spawner: MatrixMaterialSpawner,
    shape_table: QueryShapeTable,
    boards: Query<Entity, Added<PieceQueue>>,
) {
    let bounds = shape_table
        .bounds(|&ShapeParameters { rotation, .. }| rotation == RotationState::Up)
        .tap_mut(|r| *r = IRect::from_corners(IVec2::ZERO, r.size() * ivec2(1, -1)));

    let offset = MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2() / 2. * (CELL_SIZE as f32);
    let space_horiz = vec2(24., 2.);
    let space_vert = vec2(0., -(CELL_SIZE as f32 * (bounds.size().y + 1) as f32));

    for e in boards.iter() {
        let queue_sprites = (0..5)
            .map(|i| {
                let transform = (offset + space_horiz + (i as f32) * space_vert).extend(0.);
                spawner
                    .spawn(bounds)
                    .insert((Transform::from_translation(transform), QueueSprite(i)))
                    .id()
            })
            .collect_vec();

        for s in queue_sprites {
            commands.entity(e).add_child(s);
        }
    }
}

// TODO: This function does not react to changes to queue window size
// TODO: This function does not react to changes in matrix bounds
/// Updates the visual state of the piece queue. When the queue changes, each piece in the queue has
/// its texture updated to match its intended state.
pub(super) fn display_queue(
    queue: Query<(&PieceQueue, &Children), Changed<PieceQueue>>,
    mut sprites: Query<(&Handle<MatrixMaterial>, &QueueSprite)>,
    mut mats: ResMut<Assets<MatrixMaterial>>,
    shape_table: QueryShapeTable,
) {
    let bounds =
        shape_table.bounds(|&ShapeParameters { rotation, .. }| rotation == RotationState::Up);
    let matrix_size = bounds.size().x;

    for (queue, children) in queue.iter() {
        for e in children
            .iter()
            .copied()
            .filter(|&e| sprites.contains(e))
            .collect_vec()
        {
            let (mat, QueueSprite(n)) = sprites.get_mut(e).unwrap();
            let material = mats.get_mut(mat).unwrap();

            let kind = queue.window()[*n];
            let selector = ShapeParameters {
                rotation: RotationState::Up,
                kind,
            };
            let shape = &shape_table[selector];

            material.data.fill(MinoKind::E as u32);
            for &p in shape {
                let loc = p - bounds.min;
                let ix = loc.y * matrix_size + loc.x;
                material.data[ix as usize] = kind as u32;
            }
        }
    }
}
