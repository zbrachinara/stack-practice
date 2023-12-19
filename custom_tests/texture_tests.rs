use bevy::{
    app::{App, PostStartup},
    asset::{Assets, Handle, UpdateAssets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Local, Res, ResMut},
    render::texture::Image,
    sprite::{SpriteBundle, TextureAtlasBuilder},
    utils::default,
    DefaultPlugins,
};

use quickstacking::assets::{MinoPlugin, MinoTextures};

fn iter(slf: &MinoTextures) -> impl Iterator<Item = Handle<Image>> {
    [
        slf.t.clone(),
        slf.o.clone(),
        slf.l.clone(),
        slf.j.clone(),
        slf.s.clone(),
        slf.z.clone(),
        slf.i.clone(),
        slf.g.clone(),
    ]
    .into_iter()
}

fn display_each_texture(
    mut commands: Commands,
    textures: Res<MinoTextures>,
    mut texture_server: ResMut<Assets<Image>>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }

    let mut atlas_builder = TextureAtlasBuilder::default();
    for texture in iter(&textures) {
        let id = texture.id();
        let Some(texture) = texture_server.get(id) else {
            return;
        };
        atlas_builder.add_texture(id, texture);
    }

    *loaded = true;

    let texture = atlas_builder.finish(&mut texture_server).unwrap().texture;
    commands.spawn(SpriteBundle {
        texture,
        ..default()
    });
}

fn create_test_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MinoPlugin))
        .add_systems(PostStartup, create_test_camera)
        .add_systems(UpdateAssets, display_each_texture)
        .run();
}
