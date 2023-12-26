use bevy::{
    app::{App, PostStartup},
    asset::{Assets, UpdateAssets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Local, Res, ResMut},
    render::texture::Image,
    sprite::{SpriteBundle, TextureAtlasBuilder},
    utils::default,
    DefaultPlugins,
};

use stack_practice::assets::{MinoTextures, StackingAssetsPlugin};

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
    for texture in textures.iter() {
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
        .add_plugins((DefaultPlugins, StackingAssetsPlugin))
        .add_systems(PostStartup, create_test_camera)
        .add_systems(UpdateAssets, display_each_texture)
        .run();
}
