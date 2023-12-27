use bevy::{math::vec2, prelude::*, sprite::Anchor};

use crate::{
    assets::tables::{shape_table::ShapeParameters, sprite_table::SpriteTable},
    board::{Hold, RotationState, CELL_SIZE, MATRIX_DEFAULT_LEGAL_BOUNDS},
};

use super::AddedOrChanged;

#[derive(Component)]
pub struct HoldSprite;

pub(super) fn spawn_hold_sprite(mut commands: Commands, boards: Query<Entity, Added<Hold>>) {
    let hold_offset =
        MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2() / 2.0 * vec2(-1., 1.) * CELL_SIZE as f32
            - vec2(24., 2.);
    for e in boards.iter() {
        let hold_sprite = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    flip_y: true,
                    anchor: Anchor::TopRight,
                    ..default()
                },
                transform: Transform::from_translation(hold_offset.extend(0.)),
                ..default()
            })
            .insert(HoldSprite)
            .id();

        commands.entity(e).add_child(hold_sprite);
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

        if let &Hold::Ready(p) | &Hold::Inactive(p) = hold {
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
            Hold::Ready(_) => {
                *vis = Visibility::Inherited;
                spr.color = Color::WHITE;
            }
        }
    }
}
