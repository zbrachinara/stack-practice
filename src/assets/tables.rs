use bevy::{
    asset::{io::Reader, Asset, AssetEvent, AssetLoader, Assets, AsyncReadExt, LoadContext},
    ecs::{
        event::{EventReader, Events},
        system::{Commands, Res},
    },
    reflect::TypePath,
};

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
        _: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| "Could not read from the given file (when loading table)")?;
            let custom_asset =
                ron::de::from_bytes(&bytes).map_err(|_| "Could not interpret the given table")?;
            Ok(custom_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tables"]
    }
}

pub(super) fn load_tables(
    mut commands: Commands,
    mut ev: EventReader<AssetEvent<Tables>>,
    assets: Res<Assets<Tables>>,
) {
    if let Some(q) = ev.read().find_map(|p| match p {
        AssetEvent::Added { id: i } => Some(i),
        _ => None,
    }) {
        let Tables { shape } = assets.get(*q).unwrap().clone();
        println!("{shape:?}");
        commands.insert_resource(shape);
    }
}

#[derive(serde::Deserialize, Asset, TypePath, Clone)]
pub(super) struct Tables {
    shape: shape_table::ShapeTable,
}
