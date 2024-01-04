use std::fmt::Display;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, Handle, LoadContext},
    ecs::system::Resource,
    math::IVec2,
    reflect::TypePath,
    utils::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::board::{MinoKind, RotationState};

#[derive(serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[serde(from = "(MinoKind, RotationState)")]
pub struct ShapeParameters {
    pub kind: MinoKind,
    pub rotation: RotationState,
}

impl Display for ShapeParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{:?}", self.kind, self.rotation)
    }
}

impl From<(MinoKind, RotationState)> for ShapeParameters {
    fn from((kind, rotation): (MinoKind, RotationState)) -> Self {
        Self { kind, rotation }
    }
}

#[derive(serde::Deserialize, Resource, Clone, Debug, Asset, TypePath)]
pub struct ShapeTable {
    pub table: HashMap<ShapeParameters, Vec<IVec2>>,
    /// A bounding rectangle on all the coordinates listed in the table. The first coordinate is
    /// less than or equal to all coordinates in the table, and the second one is greater than or
    /// equal to all coordinates in the table.
    pub bounds: [IVec2; 2],
}

#[derive(Default)]
pub(crate) struct ShapeTableLoader;

impl AssetLoader for ShapeTableLoader {
    type Asset = ShapeTable;
    type Settings = ();
    type Error = &'static str;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a Self::Settings,
        _: &'a mut LoadContext,
        // ctx: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            tracing::debug_span!(module_path!());
            tracing::debug!("beginning shape table load");

            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| "Could not read from the given file (when loading shape table)")?;
            let shape_table: HashMap<ShapeParameters, Vec<IVec2>> = ron::de::from_bytes(&bytes)
                .map_err(|_| "Could not interpret the given shape table")?;

            // let shape_table_ref = &shape_table.0;
            // let ctx_ref = &ctx;
            // let finished_assets = join_all(all_shape_parameters().map(|params| async move {
            //     let mut ctx = ctx_ref.begin_labeled_asset();
            //     let mut new_tex = transparent_texture(uvec2(CELL_SIZE * 4, CELL_SIZE * 4));
            //     let src = ctx
            //         .load_direct(params.kind.path_of())
            //         .await
            //         .expect("could not get texture of mino")
            //         .take()
            //         .unwrap();

            //     for &p in &shape_table_ref[&params] {
            //         copy_from_to(&mut new_tex, &src, p)
            //     }

            //     let asset = ctx.finish(new_tex, None);
            //     (params, asset)
            // }))
            // .await;

            // for (name, loaded_asset) in finished_assets {
            //     ctx.add_loaded_labeled_asset(name.to_string(), loaded_asset);
            // }

            let (min, max) = shape_table
                .values()
                .flatten()
                .fold((IVec2::ZERO, IVec2::ZERO), |(a, b), &c| {
                    (a.min(c), b.max(c))
                });

            Ok(ShapeTable {
                table: shape_table,
                bounds: [min, max],
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["shape-table"]
    }
}

#[derive(Resource, AssetCollection)]
pub struct DefaultShapeTable {
    #[asset(path = "default.shape-table")]
    pub(super) table: Handle<ShapeTable>,
}
