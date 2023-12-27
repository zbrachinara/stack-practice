use bevy::prelude::*;

use crate::{
    assets::tables::{shape_table::ShapeParameters, sprite_table::SpriteTable},
    board::{Active, ActiveSprite, Bounds, CELL_SIZE},
};

use super::AddedOrChanged;

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

            *tex = sprite_table.0[&ShapeParameters::from(piece)].clone();
        } else {
            *vis = Visibility::Hidden
        }
    }
}
