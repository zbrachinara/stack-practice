use bevy::prelude::*;

use crate::{
    assets::tables::{shape_table::ShapeParameters, sprite_table::SpriteTable},
    board::{Hold, HoldSprite, RotationState},
};

use super::AddedOrChanged;

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
