use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    assets::tables::{shape_table::ShapeParameters, sprite_table::SpriteTable},
    board::{queue::PieceQueue, QueueSprite, RotationState},
};

use super::AddedOrChanged;

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
