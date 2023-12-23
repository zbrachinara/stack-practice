#![allow(clippy::type_complexity)]

use std::ops::{Deref, DerefMut};

use bevy::{
    app::{Plugin, PostUpdate, Update},
    asset::{AssetPath, Assets, Handle},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{Added, Changed, Or, With, WorldQuery},
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Children},
    math::{ivec2, vec2, IVec2, UVec2},
    render::{
        color::Color,
        render_resource::Extent3d,
        texture::Image,
        view::{InheritedVisibility, Visibility},
    },
    sprite::{Anchor, Sprite, SpriteBundle},
    transform::components::{GlobalTransform, Transform},
    utils::default,
};
use itertools::Itertools;

mod controller;
mod queue;

use crate::{
    assets::{
        tables::{shape_table::ShapeParameters, sprite_table::SpriteTable},
        MinoTextures,
    },
    state::MainState,
};

use self::{
    controller::{process_input, reset_controller, Controller},
    queue::PieceQueue,
};

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
struct Matrix {
    data: Vec<Vec<MinoKind>>,
    updates: Vec<MatrixUpdate>,
}

#[derive(Component)]
struct BoardTextures {
    matrix_cells: Handle<Image>,
}

#[derive(Component, Default)]
struct DropClock(f32);

pub fn transparent_texture(size: UVec2) -> Image {
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
#[derive(Component)]
struct QueueSprite(usize);
#[derive(Component)]
struct HoldSprite;

#[derive(Debug)]
struct MatrixUpdate {
    loc: IVec2,
    kind: MinoKind,
}

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
    queue: PieceQueue,
    drop_clock: DropClock,
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

    let hold_offset =
        MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2() / 2.0 * vec2(-1., 0.) * CELL_SIZE as f32
            + vec2(24., 2.);
    let hold_sprite = commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                flip_y: true,
                anchor: Anchor::TopRight,
                ..default()
            },
            transform: Transform::from_translation(hold_offset.extend(0.)),
            ..default()
        })
        .insert(HoldSprite)
        .id();
    let queue_sprites = (0..5)
        .map(|i| {
            let offset = MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2() / 2. * (CELL_SIZE as f32);
            let space_horiz = vec2(24., 2.);
            let space_vert = vec2(0., -(CELL_SIZE as f32 * 4.));

            let transform = (offset + space_horiz + ((i + 1) as f32) * space_vert).extend(0.);

            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        flip_y: true,
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    transform: Transform::from_translation(transform),
                    ..default()
                })
                .insert(QueueSprite(i))
                .id()
        })
        .collect_vec();

    let mut board = commands.spawn(Board {
        transform: default(),
        global_transform: default(),
        visibility: default(),
        inherited_visibility: default(),
        matrix: default(),
        bounds: default(),
        active: default(),
        hold: default(),
        queue: default(),
        drop_clock: default(),
        textures,
    });

    board
        .add_child(matrix_sprite)
        .add_child(active_sprite)
        .add_child(hold_sprite);
    for e in queue_sprites {
        board.add_child(e);
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
struct BoardQuery {
    matrix: &'static mut Matrix,
    active: &'static mut Active,
    hold: &'static mut Hold,
    queue: &'static mut PieceQueue,
    drop_clock: &'static mut DropClock,
    bounds: &'static Bounds,
}

/// Update the state of the memory-representation of the board using player input
fn update_board(
    mut boards: Query<BoardQuery, AddedOrChanged<Matrix>>,
    controller: Res<Controller>,
) {
    for mut board in boards.iter_mut() {
        if controller.hard_drop {
            todo!("Bring the piece to its lowest point, lock it, and update the board/hold/queue")
        } else if controller.soft_drop {
            todo!("Lower the piece by the amount specified by the gravity multiplier")
        }

        if controller.rotate_180 {
            todo!("Flip the piece around")
        } else if controller.rotate_left {
            todo!("Rotate piece to the left")
        } else if controller.rotate_right {
            todo!("Rotate the piece to the right")
        }

        if controller.shift_left {
            todo!("Shift the piece one position to the left")
        } else if controller.shift_right {
            todo!("Shift the piece one position to the right")
        }

        if controller.repeat_left {
            todo!("Shift the piece left by the amount specified by DAS")
        } else if controller.repeat_right {
            todo!("Shift the piece right by the amount specified by DAS")
        }

        if controller.hold {
            todo!("Attempt switching the active piece with the held piece")
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
    mut board: Query<(&BoardTextures, &mut Matrix), AddedOrChanged<Matrix>>,
    mut texture_server: ResMut<Assets<Image>>,
    mino_textures: Res<MinoTextures>,
) {
    for (textures, mut board) in board.iter_mut() {
        let mut image = texture_server
            .get(textures.matrix_cells.clone())
            .cloned()
            .unwrap();

        for up in board.updates.drain(..) {
            let tex = up.kind.select(&mino_textures);
            let replace_image = texture_server.get(tex).unwrap();
            copy_from_to(&mut image, replace_image, up.loc);
        }

        *texture_server
            .get_mut(textures.matrix_cells.clone())
            .unwrap() = image;
    }
}

type AddedOrChanged<T> = Or<(Added<T>, Changed<T>)>;

fn center_board(
    boards: Query<(&Bounds, &Children), AddedOrChanged<Bounds>>,
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
    active: Query<(&Active, &Bounds, &Children), AddedOrChanged<Active>>,
    mut sprites: Query<(&mut Visibility, &mut Transform, &mut Handle<Image>), With<ActiveSprite>>,
    sprite_table: Res<SpriteTable>,
) {
    for (Active(e), bounds, children) in active.iter() {
        let active_sprite_id = children.iter().copied().find(|&c| sprites.contains(c));
        let (mut vis, mut pos, mut tex) = sprites.get_mut(active_sprite_id.unwrap()).unwrap();

        if let Some(piece) = e {
            *vis = Visibility::Inherited;

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

// TODO: This function does not react to changes to queue window size
// TODO: This function does not react to changes in matrix bounds
/// Updates the visual state of the piece queue. When the queue changes, each piece in the queue has
/// its texture updated to match its intended state.
fn display_queue(
    queue: Query<(&PieceQueue, &Children), AddedOrChanged<PieceQueue>>,
    mut sprites: Query<(&mut Handle<Image>, &QueueSprite)>,
    sprite_table: Res<SpriteTable>,
) {
    for (queue, children) in queue.iter() {
        for e in children
            .iter()
            .copied()
            .filter(|&e| sprites.contains(e))
            .collect_vec()
        {
            let (mut tex, QueueSprite(n)) = sprites.get_mut(e).unwrap();
            let selector = ShapeParameters {
                kind: queue.window()[*n],
                rotation: RotationState::Up,
            };
            *tex = sprite_table.0[&selector].clone();
        }
    }
}

/// Displays the held piece. Greys the texture of the associated sprite if it is inactive, or keeps
/// it at its normal color if it is not. The sprite is hidden if the hold slot is empty.
fn display_held(
    hold: Query<(&Hold, &Children), AddedOrChanged<Hold>>,
    mut sprites: Query<(&mut Visibility, &mut Sprite, &mut Handle<Image>), With<HoldSprite>>,
    sprite_table: Res<SpriteTable>,
) {
    for (hold, children) in hold.iter() {
        let child = children
            .iter()
            .copied()
            .find(|&c| sprites.contains(c))
            .unwrap();
        let (mut vis, mut spr, mut tex) = sprites.get_mut(child).unwrap();

        match hold {
            &Hold::Active(p) | &Hold::Inactive(p) => {
                let selector = ShapeParameters {
                    kind: p,
                    rotation: RotationState::Up,
                };
                *tex = sprite_table.0[&selector].clone();
            }
            _ => (),
        }

        match hold {
            Hold::Empty => {
                *vis = Visibility::Hidden;
            }
            Hold::Inactive(_) => {
                spr.color = Color::GRAY;
            }
            _ => (),
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
                (
                    reset_controller,
                    center_board,
                    display_active,
                    display_queue,
                    display_held,
                    redraw_board,
                )
                    .run_if(in_state(MainState::Playing)),
            );
    }
}
