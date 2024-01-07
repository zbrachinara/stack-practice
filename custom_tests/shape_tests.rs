use bevy::math::ivec2;
use bevy::prelude::*;
use bevy::{
    math::{uvec2, vec2, Vec2},
    sprite::{ColorMaterial, MaterialMesh2dBundle},
};
use itertools::{iproduct, Itertools};
use stack_practice::assets::tables::QueryShapeTable;
use stack_practice::assets::matrix_material::MatrixMaterialSpawner;
use stack_practice::{assets::StackingAssetsPlugin, board::CELL_SIZE, state::StatePlugin};
use stack_practice::state::MainState;

fn spawn_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let base = vec2(-3.5, -1.5);
    let size = Vec2::splat(CELL_SIZE as f32 * 4.0);

    let white = materials.add(ColorMaterial::from(Color::WHITE.with_a(0.2)));
    let black = materials.add(ColorMaterial::from(Color::BLACK.with_a(0.2)));

    for (x, y) in iproduct!(0..7, 0..4) {
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
    // mut commands: Commands,
    mut camera: Query<&mut OrthographicProjection>,
    shapes: QueryShapeTable,
    mut finished: Local<bool>,
    mut spawner: MatrixMaterialSpawner,
) {
    if !*finished {
        camera.single_mut().scale = 2.0;

        shapes
            .table
            .iter()
            .sorted_by_key(|(p, _)| p.rotation)
            .map(|(p, shape)| (p.kind, shape))
            .into_group_map()
            .into_iter()
            .enumerate()
            .flat_map(|(ix, (k, a))| {
                let scale = (CELL_SIZE * 4) as f32;
                let x = (ix as f32 - 3.5) * scale;
                let ys = (0..4).map(move |p| (p as f32 - 1.5) * scale);
                let cs = ys.map(move |y| vec2(x, y));
                a.into_iter().zip(cs).zip(std::iter::repeat(k))
            })
            .for_each(|((shape, pos), kind)| {
                let mut data = vec![0; 16];
                for &s in shape {
                    let loc = s + ivec2(1, 2);
                    let ix = loc.y * 4 + loc.x;
                    data[ix as usize] = kind as u32;
                }
                spawner
                    .spawn_centered_with_data(ivec2(4, 4), data)
                    .insert(Transform::from_translation(pos.extend(0.0)));
            });

        *finished = true;
    }
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StackingAssetsPlugin, StatePlugin))
        .add_systems(Startup, (camera, spawn_grid))
        .add_systems(
            Update,
            render_all_pieces.run_if(not(in_state(MainState::Loading))),
        )
        .run();
}
