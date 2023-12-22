use bevy::{
    app::Plugin,
    asset::{AssetApp, Assets, Handle},
    ecs::system::{Res, Resource},
    render::texture::Image,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt},
};

pub mod tables;

use crate::state::MainState;

use self::tables::{SpriteTable, TableLoader, Tables};

pub struct StackingAssetsPlugin;

#[derive(Resource, AssetCollection)]
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
    pub fn iter(&self) -> impl Iterator<Item = Handle<Image>> {
        [
            self.t.clone(),
            self.o.clone(),
            self.l.clone(),
            self.j.clone(),
            self.s.clone(),
            self.z.clone(),
            self.i.clone(),
            self.g.clone(),
            self.e.clone(),
        ]
        .into_iter()
    }
}

#[derive(Resource)]
struct DefaultTables(Handle<Tables>);

/// A system that checks if mino textures have been loaded
pub fn textures_are_loaded(
    resource: Option<Res<MinoTextures>>,
    assets: Res<Assets<Image>>,
) -> bool {
    resource.is_some_and(|e| e.iter().all(|i| assets.contains(i)))
}

impl Plugin for StackingAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<Tables>()
            .add_loading_state(
                LoadingState::new(MainState::Loading).continue_to_state(MainState::Playing),
            )
            .init_asset_loader::<TableLoader>()
            .add_collection_to_loading_state::<_, MinoTextures>(MainState::Loading)
            .add_collection_to_loading_state::<_, SpriteTable>(MainState::Loading)
            // .add_systems(Startup, load_textures)
            // .add_systems(
            //     PreUpdate,
            //     (
            //         load_tables,
            //         // generate_sprites.run_if(need_sprites.and_then(textures_are_loaded)),
            //     ),
            // );
            ;
    }
}
