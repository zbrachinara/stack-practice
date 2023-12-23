use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::Resource,
    render::texture::Image,
    utils::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::assets::tables::all_shape_parameters;

use super::shape_table::ShapeParameters;

#[derive(Resource)]
pub struct SpriteTable(pub HashMap<ShapeParameters, Handle<Image>>);

impl AssetCollection for SpriteTable {
    fn create(world: &mut bevy::prelude::World) -> Self {
        tracing::info!("create called");
        let asset_server = world
            .get_resource::<AssetServer>()
            .expect("Asset server is required");
        Self(
            all_shape_parameters()
                .map(|p| (p, asset_server.load(format!("default.shape-table#{p}"))))
                .collect(),
        )
    }

    fn load(world: &mut bevy::prelude::World) -> Vec<bevy::prelude::UntypedHandle> {
        tracing::info!("load called");
        let table = world
            .get_resource::<AssetServer>()
            .expect("Asset server is required")
            .load_untyped("default.shape-table")
            .untyped();
        vec![table]
    }
}
