use bevy::{
    app::{App, PostStartup},
    asset::Handle,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Res},
    render::texture::Image,
    sprite::SpriteBundle,
    transform::components::Transform,
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

fn display_each_texture(mut commands: Commands, textures: Res<MinoTextures>) {
    for (ix, texture) in iter(&textures).enumerate() {
        let transform = Transform::from_xyz((32 * ix) as f32, 0., 0.);
        let bundle = SpriteBundle {
            texture,
            transform,
            ..default()
        };

        commands.spawn(bundle);
    }
}

fn create_test_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MinoPlugin))
        .add_systems(PostStartup, (create_test_camera, display_each_texture))
        .run();
}
