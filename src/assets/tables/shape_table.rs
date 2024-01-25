use std::fmt::Display;
use std::ops::Index;

use bevy::math::IRect;
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, Handle, LoadContext},
    ecs::system::Resource,
    math::IVec2,
    reflect::TypePath,
    utils::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::board::{Mino, MinoKind, RotationState};

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

impl From<Mino> for ShapeParameters {
    fn from(Mino { kind, rotation, .. }: Mino) -> Self {
        ShapeParameters { kind, rotation }
    }
}


#[derive(serde::Deserialize, Resource, Clone, Debug, Asset, TypePath)]
pub struct ShapeTable {
    table: HashMap<ShapeParameters, Vec<IVec2>>,
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

            Ok(ShapeTable { table: shape_table })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["shape-table"]
    }
}

impl ShapeTable {
    /// Returns a bounding rectangle on all the coordinates listed in the table. The first coordinate is
    /// less than or equal to all coordinates in the table, and the second one is greater than all
    /// coordinates in the table.
    pub fn bounds<F>(&self, mut filter: F) -> IRect
    where
        F: FnMut(&ShapeParameters) -> bool,
    {
        let (min, max) = self
            .table
            .iter()
            .filter_map(|(p, q)| filter(p).then_some(q))
            .flatten()
            .fold((IVec2::MAX, IVec2::MIN), |(a, b), &c| (a.min(c), b.max(c)));
        IRect { min, max: max + IVec2::ONE }
    }
}

impl Index<ShapeParameters> for ShapeTable {
    type Output = Vec<IVec2>;

    fn index(&self, index: ShapeParameters) -> &Self::Output {
        &self.table[&index]
    }
}

impl Index<Mino> for ShapeTable {
    type Output = Vec<IVec2>;

    fn index(&self, index: Mino) -> &Self::Output {
        &self[ShapeParameters::from(index)]
    }
}


#[derive(Resource, AssetCollection)]
pub struct DefaultShapeTable {
    #[asset(path = "default.shape-table")]
    pub(super) table: Handle<ShapeTable>,
}
