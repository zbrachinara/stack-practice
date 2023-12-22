use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AssetServer, AsyncReadExt, Handle, LoadContext},
    ecs::system::Resource,
    reflect::TypePath,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
    utils::hashbrown::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use futures::future::join_all;

use crate::board::{copy_from_to, MinoKind, RotationState, CELL_SIZE};

use self::shape_table::ShapeParameters;

pub mod shape_table;

#[derive(Default)]
pub(super) struct TableLoader;

impl AssetLoader for TableLoader {
    type Asset = Tables;
    type Settings = ();
    type Error = &'static str;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a Self::Settings,
        ctx: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            tracing::debug_span!(module_path!());

            tracing::debug!("beginning table load");

            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| "Could not read from the given file (when loading table)")?;
            let tables: Tables =
                ron::de::from_bytes(&bytes).map_err(|_| "Could not interpret the given table")?;

            let shape_table = &tables.shape.0;
            let ctx_ref = &ctx;
            let finished_assets = join_all(all_shape_parameters().map(|params| async move {
                let mut ctx = ctx_ref.begin_labeled_asset();
                let mut new_tex = Image::new_fill(
                    Extent3d {
                        width: CELL_SIZE * 4,
                        height: CELL_SIZE * 4,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    &[0, 0, 0, 0],
                    TextureFormat::Rgba8UnormSrgb,
                );
                let src = ctx
                    .load_direct(params.kind.path_of())
                    .await
                    .expect("could not get texture of mino")
                    .take()
                    .unwrap();

                for p in &shape_table[&params] {
                    copy_from_to(&mut new_tex, &src, *p)
                }

                let asset = ctx.finish(new_tex, None);
                (params, asset)
            }))
            .await;

            for (name, loaded_asset) in finished_assets {
                ctx.add_loaded_labeled_asset(name.to_string(), loaded_asset);
            }

            Ok(tables)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tables"]
    }
}

#[derive(Resource)]
pub struct SpriteTable(pub HashMap<ShapeParameters, Handle<Image>>);

/// Returns all possible shape parameters
fn all_shape_parameters() -> impl Iterator<Item = ShapeParameters> {
    use MinoKind::*;
    [T, O, L, J, S, Z, I]
        .into_iter()
        .flat_map(|kind| {
            use RotationState::*;
            [(kind, Up), (kind, Left), (kind, Down), (kind, Right)]
        })
        .map(ShapeParameters::from)
}

impl AssetCollection for SpriteTable {
    fn create(world: &mut bevy::prelude::World) -> Self {
        let asset_server = world
            .get_resource::<AssetServer>()
            .expect("Asset server is required");
        Self(
            all_shape_parameters()
                .map(|p| (p, asset_server.load(format!("default.tables#{p}"))))
                .collect(),
        )
    }

    fn load(world: &mut bevy::prelude::World) -> Vec<bevy::prelude::UntypedHandle> {
        let table = world
            .get_resource::<AssetServer>()
            .expect("Asset server is required")
            .load_untyped("default.tables")
            .untyped();
        vec![table]
    }
}

#[derive(serde::Deserialize, Asset, TypePath, Clone)]
pub(super) struct Tables {
    shape: shape_table::ShapeTable,
}

#[cfg(test)]
mod test {
    use super::all_shape_parameters;

    #[test]
    fn correct_shape_parameters() {
        assert_eq!(all_shape_parameters().count(), 28)
    }
}
