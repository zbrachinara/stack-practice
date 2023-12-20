#![allow(clippy::type_complexity)]

use bevy::{
    app::{Plugin, PostUpdate, Startup, Update},
    asset::{Assets, Handle},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{Changed, With, Added, Or},
        schedule::IntoSystemConfigs,
        system::{Commands, Local, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Children},
    math::{ivec2, IVec2, UVec2},
    render::{
        render_resource::Extent3d,
        texture::Image,
        view::{InheritedVisibility, Visibility},
    },
    sprite::{Sprite, SpriteBundle},
    transform::components::{GlobalTransform, Transform},
    utils::default,
};

mod controller;

use crate::assets::{textures_are_loaded, MinoTextures};

use self::controller::{process_input, reset_controller, Controller};

#[derive(Debug)]
#[rustfmt::skip]
enum MinoKind {
    T, O, L, J, S, Z, I, G, E
}

impl MinoKind {
    fn select(&self, textures: &Res<MinoTextures>) -> Handle<Image> {
        match self {
            MinoKind::T => &textures.t,
            MinoKind::O => &textures.o,
            MinoKind::L => &textures.l,
            MinoKind::J => &textures.j,
            MinoKind::S => &textures.s,
            MinoKind::Z => &textures.z,
            MinoKind::I => &textures.i,
            MinoKind::G => &textures.g,
            MinoKind::E => panic!(),
        }
        .clone()
    }
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

#[derive(Component, Default)]
enum Hold {
    #[default]
    Empty,
    Active(MinoKind),
    Inactive(MinoKind),
}

const MATRIX_DEFAULT_SIZE: IVec2 = ivec2(10, 40);
const MATRIX_DEFAULT_LEGAL_BOUNDS: IVec2 = ivec2(10, 20);
const CELL_SIZE: u32 = 32;

#[derive(Component)]
struct Bounds {
    true_bounds: IVec2,
    legal_bounds: IVec2,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            true_bounds: MATRIX_DEFAULT_SIZE,
            legal_bounds: MATRIX_DEFAULT_LEGAL_BOUNDS,
        }
    }
}

#[derive(Component, Default)]
struct Active(Option<Mino>);

#[derive(Component, Default)]
struct Grid(Vec<Vec<MinoKind>>);

#[derive(Bundle, Default)]
struct Matrix {
    grid: Grid,
    bounds: Bounds,
    active: Active,
    hold: Hold,
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
        let matrix_cells = transparent_texture(dimensions.as_uvec2() * CELL_SIZE);
        let matrix_cells = image_server.add(matrix_cells);
        Self { matrix_cells }
    }
}

#[derive(Component)]
struct MatrixSprite;

#[derive(Debug)]
struct MatrixUpdate {
    loc: IVec2,
    kind: MinoKind,
}

#[derive(Default, Component)]
struct MatrixUpdates(Vec<MatrixUpdate>);

#[derive(Bundle)]
pub struct Board {
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    matrix: Matrix,
    updates: MatrixUpdates,
    textures: BoardTextures,
}

fn spawn_default_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_board(mut commands: Commands, mut texture_server: ResMut<Assets<Image>>) {
    let textures = BoardTextures::init(MATRIX_DEFAULT_SIZE, &mut texture_server);

    let matrix_sprite = commands
        .spawn(SpriteBundle {
            texture: textures.matrix_cells.clone(),
            sprite: Sprite {
                flip_y: true,
                ..default()
            },
            ..default()
        })
        .insert(MatrixSprite)
        .id();

    commands
        .spawn(Board {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            inherited_visibility: default(),
            matrix: default(),
            updates: default(),
            textures,
        })
        .add_child(matrix_sprite);
}

/// Update the state of the memory-representation of the board using player input
fn update_board(
    mut board: Query<(&mut Grid, &mut MatrixUpdates)>,
    controller: Res<Controller>,
    mut activated: Local<bool>,
) {
    // TODO: Respond to controller commands
    if !*activated {
        if let Some((_, mut up)) = board.iter_mut().next() {
            *activated = true;
            up.0.push(MatrixUpdate {
                loc: ivec2(0, 0),
                kind: MinoKind::G,
            });
            up.0.push(MatrixUpdate {
                loc: ivec2(9, 19),
                kind: MinoKind::T,
            });
        }
    }
}

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
fn redraw_board(
    mut board: Query<(&BoardTextures, &mut MatrixUpdates)>,
    mut texture_server: ResMut<Assets<Image>>,
    mino_textures: Res<MinoTextures>,
) {
    for (textures, mut updates) in board.iter_mut() {
        if updates.0.is_empty() {
            continue;
        }

        let mut image = texture_server
            .get(textures.matrix_cells.clone())
            .cloned()
            .unwrap();

        for up in updates.0.drain(..) {
            let location = up.loc.as_uvec2() * CELL_SIZE;
            let width = image.width();
            let region = (location.y..location.y + CELL_SIZE).map(|col| {
                let offset = ((location.x + col * width) * 4) as usize;
                let offset_end = offset + (CELL_SIZE as usize) * 4;
                (offset, offset_end)
            });
            match up.kind {
                MinoKind::E => {
                    for (a, b) in region {
                        image.data[a..b].fill(0)
                    }
                }
                kind => {
                    let tex = kind.select(&mino_textures);
                    let replace_image = texture_server.get(tex).unwrap();
                    for (copy_region, (a, b)) in replace_image
                        .data
                        .chunks_exact(CELL_SIZE as usize * 4)
                        .zip(region)
                    {
                        image.data[a..b].copy_from_slice(copy_region);
                    }
                }
            }
        }

        *texture_server
            .get_mut(textures.matrix_cells.clone())
            .unwrap() = image;
    }
}

fn center_board(
    boards: Query<(&Bounds, &Children), Or<(Added<Bounds>, Changed<Bounds>)>>,
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

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Controller::default())
            .add_systems(Startup, (spawn_board, spawn_default_camera))
            .add_systems(Update, (process_input, update_board.after(process_input)))
            .add_systems(
                PostUpdate,
                (
                    reset_controller,
                    center_board,
                    redraw_board.run_if(textures_are_loaded),
                ),
            );
    }
}
