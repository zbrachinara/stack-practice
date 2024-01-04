use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle},
};

use crate::{
    assets::MinoTextures,
    board::{Bounds, Matrix, CELL_SIZE, MATRIX_DEFAULT_SIZE},
    image_tools::stack_images,
};

#[derive(Clone, TypePath, Asset, AsBindGroup)]
pub struct MatrixMaterial {
    #[uniform(0)]
    pub dimensions: UVec2,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub mino_textures: Handle<Image>,
    #[storage(3)]
    pub data: Vec<u32>,
}

impl Material2d for MatrixMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/matrix.wgsl".into()
    }
}

#[derive(Component)]
pub struct MatrixSprite;

pub(super) fn spawn_matrix_sprite(
    mut commands: Commands,
    boards: Query<Entity, Added<Matrix>>,
    mut texture_server: ResMut<Assets<Image>>,
    mut material_server: ResMut<Assets<MatrixMaterial>>,
    mut mesh_server: ResMut<Assets<Mesh>>,
    mino_textures: Res<MinoTextures>,
) {
    for e in boards.iter() {
        let all_textures = stack_images(&mino_textures.view(), &texture_server);
        let material = MatrixMaterial {
            dimensions: MATRIX_DEFAULT_SIZE.as_uvec2(),
            mino_textures: texture_server.add(all_textures),
            data: vec![0; 40 * 10],
        };

        let mesh_size = MATRIX_DEFAULT_SIZE.as_vec2() * (CELL_SIZE as f32);
        let matrix_sprite = commands
            .spawn(MaterialMesh2dBundle {
                mesh: mesh_server.add(shape::Quad::new(mesh_size).into()).into(),
                material: material_server.add(material),
                ..default()
            })
            .insert(MatrixSprite)
            .id();

        commands.entity(e).add_child(matrix_sprite);
    }
}

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
pub(super) fn redraw_board(
    board: Query<(&Matrix, &Bounds, &Children), Changed<Matrix>>,
    children: Query<&Handle<MatrixMaterial>, With<MatrixSprite>>,
    mut material_server: ResMut<Assets<MatrixMaterial>>,
) {
    for (board, bounds, ch) in board.iter() {
        let material_id = ch.iter().find_map(|c| children.get(*c).ok()).unwrap();
        let material = material_server.get_mut(material_id).unwrap();

        for up in board.updates.iter() {
            let ix = up.loc.y * bounds.true_bounds.x + up.loc.x;
            material.data[ix as usize] = up.kind as u32;
        }
    }
}

/// Centers the legal part of the matrix rather than the entire matrix.
pub(super) fn center_board(
    boards: Query<(&Bounds, &Children), Changed<Bounds>>,
    mut sprites: Query<&mut Transform, With<MatrixSprite>>,
) {
    for (board, children) in boards.iter() {
        let board_bounds = board.true_bounds.as_vec2();
        let legal_bounds = board.legal_bounds.as_vec2();
        let offset = (board_bounds / 2. - legal_bounds / 2.) * (CELL_SIZE as f32);

        let child = *children.iter().find(|q| sprites.contains(**q)).unwrap();
        sprites.get_mut(child).unwrap().translation = offset.extend(0.0);
    }
}
