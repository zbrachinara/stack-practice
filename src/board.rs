use bevy::{
    app::{Plugin, PostUpdate, Startup, Update},
    asset::{Assets, Handle},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    math::{ivec2, IVec2, UVec2},
    render::{render_resource::Extent3d, texture::Image},
    transform::components::Transform,
    utils::HashMap,
};

mod controller;

use self::controller::{process_input, reset_controller, Controller};

#[rustfmt::skip]
enum MinoKind {
    T, O, L, J, S, Z, I, G, E
}

#[derive(Default)]
#[rustfmt::skip]
enum RotationState {
    #[default] Up, Right, Down, Left
}

struct Mino {
    kind: MinoKind,
    translation: IVec2,
    rotation: RotationState,
}

#[derive(Default)]
enum Hold {
    #[default]
    Empty,
    Active(MinoKind),
    Inactive(MinoKind),
}

const MATRIX_DEFAULT_SIZE: IVec2 = ivec2(10, 40);

#[derive(Component)]
struct Matrix {
    grid: Vec<Vec<MinoKind>>,
    bounds: IVec2,
    active: Option<Mino>,
    hold: Hold,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            grid: Default::default(),
            bounds: MATRIX_DEFAULT_SIZE,
            active: Default::default(),
            hold: Default::default(),
        }
    }
}

#[derive(Component)]
struct BoardTextures {
    matrix_cells: Handle<Image>,
}

fn transparent_texture(size: UVec2) -> Image {
    let mut img = Image::default();
    img.data.fill(0);
    img.resize(Extent3d {
        width: size.x,
        height: size.y,
        depth_or_array_layers: 1,
    });
    img
}

impl BoardTextures {
    /// Initialize textures representing an empty board
    fn init(dimensions: IVec2, image_server: &mut Assets<Image>) -> Self {
        let matrix_cells = transparent_texture(dimensions.as_uvec2());
        let matrix_cells = image_server.add(matrix_cells);
        Self { matrix_cells }
    }
}

enum MatrixUpdate {
    Empty { loc: IVec2 },
    Update { loc: IVec2, kind: MinoKind },
}

#[derive(Default, Component)]
struct MatrixUpdates(Vec<MatrixUpdate>);

#[derive(Bundle)]
pub struct Board {
    transform: Transform,
    matrix: Matrix,
    updates: MatrixUpdates,
    textures: BoardTextures,
}

fn spawn_board(mut commands: Commands, mut texture_server: ResMut<Assets<Image>>) {
    commands.spawn(Board {
        transform: Default::default(),
        matrix: Default::default(),
        updates: Default::default(),
        textures: BoardTextures::init(MATRIX_DEFAULT_SIZE, &mut texture_server),
    });
}

/// Update the state of the memory-representation of the board using player input
fn update_board(board: Query<(&mut Matrix, &mut MatrixUpdates)>, controller: Res<Controller>) {
    unimplemented!()
}

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
fn redraw_board(board: Query<(&BoardTextures, &mut MatrixUpdates)>) {
    unimplemented!()
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Controller::default())
            .add_systems(Startup, spawn_board)
            .add_systems(Update, (process_input, update_board.after(process_input)))
            .add_systems(PostUpdate, (reset_controller, redraw_board));
    }
}
