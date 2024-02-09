use bevy::prelude::*;

use crate::{
    assets::tables::QueryShapeTable,
    board::{Active, Bounds, MinoKind, CELL_SIZE},
};

use crate::assets::matrix_material::{MatrixMaterial, MatrixMaterialSpawner};

#[derive(Component)]
pub struct ActiveSprite;

pub(crate) fn spawn_active_sprite(
    mut commands: Commands,
    boards: Query<Entity, Added<Active>>,
    mut mat_spawner: MatrixMaterialSpawner,
    shape_table: QueryShapeTable,
) {
    for e in boards.iter() {
        let active_sprite = mat_spawner
            .spawn(shape_table.bounds(|_| true))
            .insert(ActiveSprite)
            .id();

        commands.entity(e).add_child(active_sprite);
    }
}

/// Updates the visual state of the active piece. The active piece is a child of the board,
/// initialized in the same system that spawns the board. If the active piece becomes `None`, then
/// the sprite representing it is hidden. If it is modified in any other way, the sprite's position
/// and kind will be updated to match.
pub(crate) fn display_active(
    active: Query<(&Active, &Bounds, &Children), Changed<Active>>,
    mut sprites: Query<
        (&mut Visibility, &mut Transform, &Handle<MatrixMaterial>),
        With<ActiveSprite>,
    >,
    shape_table: QueryShapeTable,
    mut material_server: ResMut<Assets<MatrixMaterial>>,
) {
    let shape_bounds = shape_table.bounds(|_| true);
    for (Active(e), bounds, children) in active.iter() {
        let active_sprite_id = children.iter().copied().find(|&c| sprites.contains(c));
        let (mut vis, mut pos, tex) = sprites.get_mut(active_sprite_id.unwrap()).unwrap();
        let mat = material_server.get_mut(tex).unwrap();

        if let Some(piece) = e {
            *vis = Visibility::Inherited;

            let offset = -(bounds.legal_bounds.as_vec2() / 2.);
            let new_pos = (piece.position.as_vec2() + offset) * CELL_SIZE as f32;
            pos.translation = new_pos.extend(1.0);

            mat.data.fill(MinoKind::E as u32);
            let shape = &shape_table[*piece];
            for &p in shape {
                let loc = p - shape_bounds.min;
                let ix = loc.y * (shape_bounds.size().x) + loc.x;
                mat.data[ix as usize] = piece.kind as u32;
            }
        } else {
            *vis = Visibility::Hidden
        }
    }
}
