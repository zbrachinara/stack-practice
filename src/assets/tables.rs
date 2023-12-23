use std::ops::Deref;

use bevy::{
    asset::Assets,
    ecs::system::{Res, SystemParam},
};

use self::{
    kick_table::{DefaultKickTable, KickTable},
    shape_table::{DefaultShapeTable, ShapeParameters, ShapeTable},
};

pub mod kick_table;
pub mod shape_table;
pub mod sprite_table;

/// Returns all possible shape parameters
fn all_shape_parameters() -> impl Iterator<Item = ShapeParameters> {
    use crate::board::MinoKind::*;
    [T, O, L, J, S, Z, I]
        .into_iter()
        .flat_map(|kind| {
            use crate::board::RotationState::*;
            [(kind, Up), (kind, Left), (kind, Down), (kind, Right)]
        })
        .map(ShapeParameters::from)
}

duplicate::duplicate! {
    [
n default t;
[QueryShapeTable] [DefaultShapeTable] [ShapeTable];
[QueryKickTable]  [DefaultKickTable]  [KickTable] ;
    ]

    #[derive(SystemParam)]
    pub struct n<'w> {
        table: Res<'w, default>,
        assets: Res<'w, Assets<t>>,
    }

    impl<'w> Deref for n<'w> {
        type Target = t;

        fn deref(&self) -> &Self::Target {
            self.assets.get(self.table.table.clone()).unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use super::all_shape_parameters;

    #[test]
    fn correct_shape_parameters() {
        assert_eq!(all_shape_parameters().count(), 28)
    }
}
