use bevy::{
    asset::{Asset, AssetLoader, AsyncReadExt, Handle},
    ecs::system::Resource,
    math::IVec2,
    reflect::TypePath,
    utils::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::board::{MinoKind, RotationState};

#[derive(serde::Deserialize, PartialEq, Eq, Hash)]
#[serde(from = "(MinoKind, RotationState, RotationState)")]
pub struct KickParameters {
    pub kind: MinoKind,
    pub from: RotationState,
    pub to: RotationState,
}

impl From<(MinoKind, RotationState, RotationState)> for KickParameters {
    fn from((kind, from, to): (MinoKind, RotationState, RotationState)) -> Self {
        Self { kind, from, to }
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct KickTable(pub HashMap<KickParameters, Vec<IVec2>>);

#[derive(Default)]
pub struct KickTableLoader;
impl AssetLoader for KickTableLoader {
    type Asset = KickTable;
    type Settings = ();
    type Error = &'static str;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _: &'a Self::Settings,
        _: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| "Could not read from the given file (when loading kick table)")?;
            ron::de::from_bytes::<KickTable>(&bytes)
                .map_err(|e| {println!("{}", e); "Could not interpret the given kick table"})
        })
    }

    fn extensions(&self) -> &[&str] {
        &["kick-table"]
    }
}

#[derive(Resource, AssetCollection)]
pub struct DefaultKickTable {
    #[asset(path = "default.kick-table")]
    pub(super) table: Handle<KickTable>,
}
