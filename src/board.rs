#![allow(clippy::type_complexity)]

use bevy::{
    app::{Plugin, PostUpdate, Update},
    asset::{AssetPath, Assets, Handle},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{Added, Changed, Or, With},
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter},
        system::{Commands, Local, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Children},
    math::{ivec2, IVec2, UVec2},
    render::{
        render_resource::Extent3d,
        texture::Image,
        view::{InheritedVisibility, Visibility},
    },
    sprite::{Anchor, Sprite, SpriteBundle},
    transform::components::{GlobalTransform, Transform},
    utils::default,
};

mod controller;

use crate::{
    assets::{
        tables::{shape_table::ShapeParameters, SpriteTable},
        MinoTextures,
    },
    state::MainState,
};

use self::controller::{process_input, reset_controller, Controller};

#[derive(Debug, PartialEq, Eq, Hash, serde::Deserialize, Clone, Copy)]
#[rustfmt::skip]
pub enum MinoKind {
    T, O, L, J, S, Z, I, G, E
}

impl MinoKind {
    pub fn select(&self, textures: &MinoTextures) -> Handle<Image> {
        match self {
            MinoKind::T => &textures.t,
            MinoKind::O => &textures.o,
            MinoKind::L => &textures.l,
            MinoKind::J => &textures.j,
            MinoKind::S => &textures.s,
            MinoKind::Z => &textures.z,
            MinoKind::I => &textures.i,
            MinoKind::G => &textures.g,
            MinoKind::E => &textures.e,
        }
        .clone()
    }

    pub fn path_of(&self) -> AssetPath {
        match self {
            MinoKind::T => "minos/T.png".into(),
            MinoKind::O => "minos/O.png".into(),
            MinoKind::L => "minos/L.png".into(),
            MinoKind::J => "minos/J.png".into(),
            MinoKind::S => "minos/S.png".into(),
            MinoKind::Z => "minos/Z.png".into(),
            MinoKind::I => "minos/I.png".into(),
            MinoKind::G => "minos/G.png".into(),
            MinoKind::E => "minos/E.png".into(),
        }
    }
}

#[derive(Default, PartialEq, Eq, Hash, serde::Deserialize, Clone, Copy, Debug, PartialOrd, Ord)]
#[rustfmt::skip]
pub enum RotationState {
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
/// The amount by which the spawn location of the piece is offset from the bottom left corner of its
/// texture. This should be uniform for all pieces, hence why it is declared constant here.
const TEXTURE_CENTER_OFFSET: IVec2 = ivec2(1, 2);
pub const CELL_SIZE: u32 = 32;

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
struct Matrix(Vec<Vec<MinoKind>>);

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
#[derive(Component)]
struct ActiveSprite;

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
    bounds: Bounds,
    active: Active,
    hold: Hold,
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
    let active_sprite = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                flip_y: true,
                anchor: Anchor::BottomLeft,
                ..default()
            },
            ..default()
        })
        .insert(ActiveSprite)
        .id();

    commands
        .spawn(Board {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            inherited_visibility: default(),
            matrix: default(),
            bounds: default(),
            active: default(),
            hold: default(),
            updates: default(),
            textures,
        })
        .add_child(matrix_sprite)
        .add_child(active_sprite);
}

/// Update the state of the memory-representation of the board using player input
fn update_board(
    mut board: Query<(&mut Matrix, &mut MatrixUpdates)>,
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

/// This function FLIPS the image of `src` in the y direction, and it also flips `location` in the y
/// direction relative to standard bevy coordinates (that is, y points down).
///
/// Copies data from `src` into a region in `dst`. The region is described by `location`. It is
/// interpreted as a square with length `CELL_SIZE`, positioned at the given coordinate *after*
/// scaled by `CELL_SIZE`.
///
/// Essentially each image is treated as a grid, and one grid square is copied from src to dst.
pub(crate) fn copy_from_to(dst: &mut Image, src: &Image, location: IVec2) {
    let width = dst.width();
    let location = location.as_uvec2() * CELL_SIZE;
    let region = (location.y..location.y + CELL_SIZE).map(|col| {
        let offset = ((location.x + col * width) * 4) as usize;
        let offset_end = offset + (CELL_SIZE as usize) * 4;
        (offset, offset_end)
    });

    src.data
        .chunks_exact(CELL_SIZE as usize * 4)
        .zip(region)
        .for_each(|(src, (a, b))| {
            dst.data[a..b].copy_from_slice(src);
        })
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
            let tex = up.kind.select(&mino_textures);
            let replace_image = texture_server.get(tex).unwrap();
            copy_from_to(&mut image, replace_image, up.loc);
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

/// Updates the visual state of the active piece. The active piece is a child of the board,
/// initialized in the same system that spawns the board. If the active pice becomes `None`, then
/// the sprite representing it is hidden. If it is modified in any other way, the sprite's position
/// and kind will be updated to match.
fn display_active(
    active: Query<(&Active, &Bounds, &Children), Or<(Added<Active>, Changed<Active>)>>,
    mut sprites: Query<(&mut Visibility, &mut Transform, &mut Handle<Image>), With<ActiveSprite>>,
    sprite_table: Res<SpriteTable>,
) {
    for (Active(e), bounds, children) in active.iter() {
        let active_sprite_id = children.iter().copied().find(|&c| sprites.contains(c));
        let (mut vis, mut pos, mut tex) = sprites.get_mut(active_sprite_id.unwrap()).unwrap();

        if let Some(piece) = e {
            let offset = -(bounds.true_bounds.as_vec2() / 2. + TEXTURE_CENTER_OFFSET.as_vec2());
            let new_pos = (piece.translation.as_vec2() + offset) * CELL_SIZE as f32;
            pos.translation = new_pos.extend(1.0);

            *tex = sprite_table.0[&ShapeParameters {
                kind: piece.kind,
                rotation: piece.rotation,
            }]
                .clone();
        } else {
            *vis = Visibility::Hidden
        }
    }
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Controller::default())
            .add_systems(
                OnEnter(MainState::Playing),
                (spawn_board, spawn_default_camera),
            )
            .add_systems(
                Update,
                (process_input, update_board.after(process_input))
                    .run_if(in_state(MainState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (reset_controller, center_board, display_active, redraw_board)
                    .run_if(in_state(MainState::Playing)),
            );
    }
}
