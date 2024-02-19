use bevy::sprite::Material2dPlugin;
use bevy::{
    app::Plugin,
    asset::{AssetApp, Handle},
    ecs::system::Resource,
    render::texture::Image,
};
use bevy_asset_loader::prelude::ConfigureLoadingState;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt},
};

mod image_tools;
pub mod matrix_material;
pub mod tables;

use crate::assets::matrix_material::MatrixMaterial;
use crate::state::MainState;

use self::tables::{
    kick_table::{DefaultKickTable, KickTable, KickTableLoader},
    shape_table::{DefaultShapeTable, ShapeTable, ShapeTableLoader},
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
}

impl Plugin for StackingAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(Material2dPlugin::<MatrixMaterial>::default())
            .init_asset::<ShapeTable>()
            .init_asset::<KickTable>()
            .add_loading_state(
                LoadingState::new(MainState::Loading)
                    .continue_to_state(MainState::Ready)
                    .load_collection::<MinoTextures>()
                    .load_collection::<DefaultShapeTable>()
                    .load_collection::<DefaultKickTable>(),
            )
            .init_asset_loader::<ShapeTableLoader>()
            .init_asset_loader::<KickTableLoader>();
    }
}
