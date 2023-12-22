use bevy::{
    app::{App, Startup, Update},
    asset::Assets,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::{Commands, Local, Query, Res, ResMut},
    math::{uvec2, vec2, Vec2},
    render::{
        camera::OrthographicProjection,
        color::Color,
        mesh::{shape, Mesh},
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle},
    transform::components::Transform,
    utils::default,
    DefaultPlugins,
};
use itertools::{iproduct, Itertools};
use quickstacking::{
    assets::{tables::SpriteTable, MinoPlugin},
    board::CELL_SIZE,
};

fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let base = vec2(-3.5, -1.5);
    let size = Vec2::splat(CELL_SIZE as f32 * 4.0);

    let white = materials.add(ColorMaterial::from(Color::WHITE.with_a(0.2)));
    let black = materials.add(ColorMaterial::from(Color::BLACK.with_a(0.2)));

    for (x, y) in iproduct!((0..7), (0..4)) {
        let p = (uvec2(x, y).as_vec2() + base) * size;
        let parity = (x + y) % 2 == 0;
        commands.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(size).into()).into(),
            material: if parity { white.clone() } else { black.clone() },
            transform: Transform::from_translation(p.extend(-1.0)),
            ..default()
        });
    }
}

fn render_all_pieces(
    mut commands: Commands,
    mut camera: Query<&mut OrthographicProjection>,
    sprites: Option<Res<SpriteTable>>,
    mut finished: Local<bool>,
) {
    if !*finished {
        if let Some(sprites) = sprites {
            camera.single_mut().scale = 2.0;

            sprites
                .0
                .iter()
                .sorted_by_key(|(p, _)| p.rotation)
                .map(|(p, i)| (p.kind, i))
                .into_group_map()
                .into_iter()
                .enumerate()
                .flat_map(|(ix, (_, a))| {
                    let scale = (CELL_SIZE * 4) as f32;
                    let x = (ix as f32 - 3.5) * scale;
                    let ys = (0..4).map(move |p| (p as f32 - 1.5) * scale);
                    let cs = ys.map(move |y| vec2(x, y));
                    a.into_iter().zip(cs)
                })
                .for_each(|(tex, pos)| {
                    commands.spawn(SpriteBundle {
                        transform: Transform::from_translation(pos.extend(0.0)),
                        texture: tex.clone(),
                        sprite: Sprite {
                            flip_y: true,
                            ..default()
                        },
                        ..default()
                    });
                });

            *finished = true;
        }
    }
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MinoPlugin))
        .add_systems(Startup, (camera, spawn_grid))
        .add_systems(Update, render_all_pieces)
        .run();
}
