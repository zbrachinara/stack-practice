use bevy::{
    app::{Plugin, PreUpdate, Startup},
    asset::{
        io::Reader, AssetApp, AssetLoader, AssetServer, Assets, AsyncReadExt, Handle, LoadContext,
    },
    ecs::{
        schedule::{Condition, IntoSystemConfigs},
        system::{Commands, Local, Res, ResMut, Resource},
    },
    render::texture::Image,
};

pub mod tables;

use tables::{shape_table::ShapeTable, Tables};

use self::tables::{load_tables, TableLoader};

pub struct MinoPlugin;

#[derive(Resource)]
pub struct MinoTextures {
    pub t: Handle<Image>,
    pub o: Handle<Image>,
    pub l: Handle<Image>,
    pub j: Handle<Image>,
    pub s: Handle<Image>,
    pub z: Handle<Image>,
    pub i: Handle<Image>,
    pub g: Handle<Image>,
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
        ]
        .into_iter()
    }
}

#[derive(Resource)]
struct DefaultTables(Handle<Tables>);

fn load_textures(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let t = asset_server.load("minos/T.png");
    let o = asset_server.load("minos/O.png");
    let l = asset_server.load("minos/L.png");
    let j = asset_server.load("minos/J.png");
    let s = asset_server.load("minos/S.png");
    let z = asset_server.load("minos/Z.png");
    let i = asset_server.load("minos/I.png");
    let g = asset_server.load("minos/G.png");

    #[rustfmt::skip]
    let textures = MinoTextures { t, o, l, j, s, z, i, g };

    commands.insert_resource(textures);
    commands.insert_resource(DefaultTables(
        asset_server.load::<Tables>("default.tables"),
    ));
}

fn shape_table_needs_loading(res: Option<Res<ShapeTable>>) -> bool {
    res.is_none()
}

/// A system that checks if mino textures have been loaded
pub fn textures_are_loaded(
    resource: Option<Res<MinoTextures>>,
    assets: Res<Assets<Image>>,
) -> bool {
    resource.is_some_and(|e| e.iter().all(|i| assets.contains(i)))
}

impl Plugin for MinoPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<Tables>()
            .init_asset_loader::<TableLoader>()
            .add_systems(Startup, load_textures)
            .add_systems(PreUpdate, load_tables);
    }
}
