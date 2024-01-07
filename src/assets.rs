use bevy::sprite::Material2dPlugin;
use bevy::{
    app::Plugin,
    asset::{AssetApp, Handle},
    ecs::system::Resource,
    render::texture::Image,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt},
};
use strum::IntoEnumIterator;

pub mod matrix_material;
pub mod tables;

use crate::{board::MinoKind, state::MainState};
use crate::assets::matrix_material::MatrixMaterial;

use self::tables::{
    kick_table::{DefaultKickTable, KickTable, KickTableLoader},
    shape_table::{DefaultShapeTable, ShapeTable, ShapeTableLoader},
    sprite_table::SpriteTable,
};

pub struct StackingAssetsPlugin;

#[derive(Resource, AssetCollection, Clone)]
pub struct MinoTextures {
    #[asset(path = "minos/T.png")]
    pub t: Handle<Image>,
    #[asset(path = "minos/O.png")]
    pub o: Handle<Image>,
    #[asset(path = "minos/L.png")]
    pub l: Handle<Image>,
    #[asset(path = "minos/J.png")]
    pub j: Handle<Image>,
    #[asset(path = "minos/S.png")]
    pub s: Handle<Image>,
    #[asset(path = "minos/Z.png")]
    pub z: Handle<Image>,
    #[asset(path = "minos/I.png")]
    pub i: Handle<Image>,
    #[asset(path = "minos/G.png")]
    pub g: Handle<Image>,
    #[asset(path = "minos/E.png")]
    pub e: Handle<Image>,
}

impl MinoTextures {
    pub fn view(&self) -> [Handle<Image>; 9] {
        [
            self.e.clone(),
            self.t.clone(),
            self.o.clone(),
            self.l.clone(),
            self.j.clone(),
            self.s.clone(),
            self.z.clone(),
            self.i.clone(),
            self.g.clone(),
        ]
    }
    pub fn iter_with_kind(&self) -> impl Iterator<Item = (MinoKind, Handle<Image>)> + '_ {
        MinoKind::iter().map(|i| (i, i.select(self)))
    }
}

impl Plugin for StackingAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((Material2dPlugin::<MatrixMaterial>::default(),))
            .init_asset::<ShapeTable>()
            .init_asset::<KickTable>()
            .add_loading_state(
                LoadingState::new(MainState::Loading).continue_to_state(MainState::Ready),
            )
            .init_asset_loader::<ShapeTableLoader>()
            .init_asset_loader::<KickTableLoader>()
            .add_collection_to_loading_state::<_, MinoTextures>(MainState::Loading)
            .add_collection_to_loading_state::<_, SpriteTable>(MainState::Loading)
            .add_collection_to_loading_state::<_, DefaultShapeTable>(MainState::Loading)
            .add_collection_to_loading_state::<_, DefaultKickTable>(MainState::Loading);
    }
}
