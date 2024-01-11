use std::ops::Deref;

use bevy::{
    asset::Assets,
    ecs::system::{Res, SystemParam},
};

use self::{
    kick_table::{DefaultKickTable, KickTable},
    shape_table::{DefaultShapeTable, ShapeTable},
};

pub mod kick_table;
pub mod shape_table;

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
