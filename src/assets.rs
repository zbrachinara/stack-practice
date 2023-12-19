use bevy::{
    app::{Plugin, Startup},
    asset::{AssetServer, Handle},
    ecs::system::{Commands, ResMut, Resource, Res},
    render::texture::Image,
};

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
}

/// A system that checks if mino textures have been loaded
pub fn textures_are_loaded(resource: Option<Res<MinoTextures>>) -> bool {
    resource.is_some()
}

impl Plugin for MinoPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, load_textures);
    }
}
