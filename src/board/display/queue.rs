use bevy::{math::vec2, prelude::*, sprite::Anchor};
use itertools::Itertools;

use crate::{
    assets::tables::{shape_table::ShapeParameters, sprite_table::SpriteTable},
    board::{queue::PieceQueue, RotationState, CELL_SIZE, MATRIX_DEFAULT_LEGAL_BOUNDS},
};

use super::AddedOrChanged;

#[derive(Component)]
pub struct QueueSprite(usize);

pub(super) fn spawn_queue_sprite(mut commands: Commands, boards: Query<Entity, Added<PieceQueue>>) {
    let offset = MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2() / 2. * (CELL_SIZE as f32);
    let space_horiz = vec2(24., 2.);
    let space_vert = vec2(0., -(CELL_SIZE as f32 * 4.));

    for e in boards.iter() {
        let queue_sprites = (0..5)
            .map(|i| {
                let transform = (offset + space_horiz + ((i + 1) as f32) * space_vert).extend(0.);
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            flip_y: true,
                            anchor: Anchor::BottomLeft,
                            ..default()
                        },
                        transform: Transform::from_translation(transform),
                        ..default()
                    })
                    .insert(QueueSprite(i))
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
