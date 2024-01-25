use bevy::{math::vec2, prelude::*};
use tap::Tap;

use crate::assets::matrix_material::{MatrixMaterial, MatrixMaterialSpawner};
use crate::assets::tables::QueryShapeTable;
use crate::board::MinoKind;
use crate::{
    assets::tables::shape_table::ShapeParameters,
    board::{Hold, RotationState, CELL_SIZE, MATRIX_DEFAULT_LEGAL_BOUNDS},
};

#[derive(Component)]
pub struct HoldSprite;

pub(super) fn spawn_hold_sprite(
    mut commands: Commands,
    boards: Query<Entity, Added<Hold>>,
    shape_table: QueryShapeTable,
    mut spawner: MatrixMaterialSpawner,
) {
    let hold_offset =
        MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2() / 2.0 * vec2(-1., 1.) * CELL_SIZE as f32;
    let bounds = shape_table
        .bounds(|&ShapeParameters { rotation, .. }| rotation == RotationState::Up)
        .tap_mut(|r| {
            r.min = -r.size();
            r.max = IVec2::ZERO;
        });

    for e in boards.iter() {
        let hold_sprite = spawner
            .spawn(bounds)
            .insert((
                Transform::from_translation(hold_offset.extend(0.)),
                HoldSprite,
            ))
            .id();

        commands.entity(e).add_child(hold_sprite);
    }
}

/// Displays the held piece. Greys the texture of the associated sprite if it is inactive, or keeps
/// it at its normal color if it is not. The sprite is hidden if the hold slot is empty.
pub(super) fn display_held(
    hold: Query<(&Hold, &Children), Changed<Hold>>,
    shape_table: QueryShapeTable,
    mut sprites: Query<(&mut Visibility, &Handle<MatrixMaterial>), With<HoldSprite>>,
    mut mats: ResMut<Assets<MatrixMaterial>>,
) {
    let bounds =
        shape_table.bounds(|&ShapeParameters { rotation, .. }| rotation == RotationState::Up);
    let matrix_size = bounds.size().x;
    for (hold, children) in hold.iter() {
        let child = children
            .iter()
            .copied()
            .find(|&c| sprites.contains(c))
            .unwrap();
        let (mut vis, han) = sprites.get_mut(child).unwrap();
        let mat = mats.get_mut(han).unwrap();

        match hold {
            Hold::Empty => {
                *vis = Visibility::Hidden;
            }
            &Hold::Inactive(kind) | &Hold::Ready(kind) => {
                mat.data.fill(MinoKind::E as u32);

                let shape = &shape_table[ShapeParameters {
                    kind,
                    rotation: RotationState::Up,
                }];
                for &p in shape {
                    let fill_kind = if matches!(hold, Hold::Inactive(_)) {
                        MinoKind::G
                    } else {
                        kind
                    };

                    let loc = p - bounds.min;
                    let ix = loc.y * matrix_size + loc.x;
                    mat.data[ix as usize] = fill_kind as u32;
                }

                *vis = Visibility::Inherited;
            }
        }
    }
}
