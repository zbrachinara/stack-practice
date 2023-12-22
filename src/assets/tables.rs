use bevy::{
    asset::{
        io::Reader, Asset, AssetEvent, AssetLoader, Assets, AsyncReadExt, Handle, LoadContext,
    },
    ecs::{
        event::EventReader,
        system::{Commands, Res, ResMut, Resource},
    },
    reflect::TypePath,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
    utils::hashbrown::HashMap,
};

use crate::board::{copy_from_to, MinoKind, RotationState, CELL_SIZE};

use self::shape_table::{ShapeParameters, ShapeTable};

use super::MinoTextures;

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

#[derive(Resource, Asset, TypePath)]
pub struct SpriteTable(pub HashMap<ShapeParameters, Handle<Image>>);

fn generate_sprite(shape_table: &ShapeTable, source: &Image, params: ShapeParameters) -> Image {
    let positions = shape_table.0.get(&params).unwrap();
    let mut tex = Image::new_fill(
        Extent3d {
            width: CELL_SIZE * 4,
            height: CELL_SIZE * 4,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
    );

    for p in positions {
        copy_from_to(&mut tex, source, *p);
    }

    tex
}

pub(super) fn generate_sprites(
    mut commands: Commands,
    shape_table: Res<ShapeTable>,
    textures: Res<MinoTextures>,
    mut assets: ResMut<Assets<Image>>,
) {
    use MinoKind::*;
    let sprite_table = [T, O, L, J, S, Z, I]
        .into_iter()
        .flat_map(|kind| {
            use RotationState::*;
            [(kind, Up), (kind, Left), (kind, Down), (kind, Right)]
        })
        .map(|(kind, rotation)| {
            let params = ShapeParameters { kind, rotation };
            let src = assets.get(params.kind.select(&textures)).unwrap();
            let tex = generate_sprite(&shape_table, src, params);
            (params, assets.add(tex))
        })
        .collect();

    commands.insert_resource(SpriteTable(sprite_table))
}

pub fn need_sprites(r1: Option<Res<ShapeTable>>, r2: Option<Res<SpriteTable>>) -> bool {
    r1.is_some() && r2.is_none()
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

        commands.insert_resource(shape);
    }
}

#[derive(serde::Deserialize, Asset, TypePath, Clone)]
pub(super) struct Tables {
    shape: shape_table::ShapeTable,
}
