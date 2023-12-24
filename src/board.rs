#![allow(clippy::type_complexity)]

use std::ops::Deref;

use bevy::{
    app::{Plugin, PostUpdate, Update},
    asset::{AssetPath, Assets, Handle},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::WorldQuery,
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    math::{ivec2, vec2, IVec2, UVec2},
    render::{
        camera::OrthographicProjection,
        render_resource::Extent3d,
        texture::Image,
        view::{InheritedVisibility, Visibility},
    },
    sprite::{Anchor, Sprite, SpriteBundle},
    transform::components::{GlobalTransform, Transform},
    utils::default,
};
use itertools::Itertools;
use tap::Tap;

mod controller;
mod display;
mod queue;

use crate::{
    assets::{
        tables::{
            kick_table::KickParameters,
            shape_table::{ShapeParameters, ShapeTable},
            QueryKickTable, QueryShapeTable,
        },
        MinoTextures,
    },
    state::MainState,
};

use self::{
    controller::{process_input, reset_controller, Controller},
    display::{center_board, display_active, display_held, display_queue, redraw_board},
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

impl RotationState {
    fn rotate_180(self) -> Self {
        use RotationState::*;
        match self {
            Up => Down,
            Right => Left,
            Down => Up,
            Left => Right,
        }
    }

    fn rotate_left(self) -> Self {
        use RotationState::*;
        match self {
            Up => Left,
            Right => Up,
            Down => Right,
            Left => Down,
        }
    }

    fn rotate_right(self) -> Self {
        use RotationState::*;
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }
}

#[derive(Clone, Copy)]
struct Mino {
    kind: MinoKind,
    position: IVec2,
    rotation: RotationState,
}

#[derive(Component, Default)]
enum Hold {
    #[default]
    Empty,
    Active(MinoKind),
    Inactive(MinoKind),
}

impl Hold {
    fn activate(&mut self) {
        if let Self::Inactive(p) = self {
            *self = Self::Active(*p);
        }
    }
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

#[derive(Component)]
struct Matrix {
    data: Vec<Vec<MinoKind>>,
    updates: Vec<MatrixUpdate>,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            data: std::iter::repeat_with(|| vec![MinoKind::E; MATRIX_DEFAULT_SIZE.x as usize])
                .take(MATRIX_DEFAULT_SIZE.y as usize)
                .collect(),
            updates: Default::default(),
        }
    }
}

#[derive(Component)]
struct BoardTextures {
    matrix_cells: Handle<Image>,
}

#[derive(Component, Default)]
struct DropClock(f32);

impl Matrix {
    fn get(&self, ix: IVec2) -> Option<MinoKind> {
        if ix.cmpge(ivec2(0, 0)).all() {
            self.data
                .get(ix.y as usize)
                .and_then(|row| row.get(ix.x as usize))
                .copied()
        } else {
            None
        }
    }

    fn get_mut(&mut self, ix: IVec2) -> Option<&mut MinoKind> {
        if ix.cmpge(ivec2(0, 0)).all() {
            self.data
                .get_mut(ix.y as usize)
                .and_then(|row| row.get_mut(ix.x as usize))
        } else {
            None
        }
    }
}

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

fn set_camera_scale(mut camera: Query<&mut OrthographicProjection>) {
    camera.single_mut().scale = 2.0;
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
        MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2() / 2.0 * vec2(-1., 1.) * CELL_SIZE as f32
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

impl From<&Mino> for ShapeParameters {
    fn from(&Mino { kind, rotation, .. }: &Mino) -> Self {
        ShapeParameters { kind, rotation }
    }
}

/// Checks if the matrix can accomodate the given piece.
fn has_free_space(matrix: &Matrix, mino: Mino, shape_table: &ShapeTable) -> bool {
    shape_table.0[&ShapeParameters::from(&mino)]
        .iter()
        .map(|&shape_offset| shape_offset + mino.position)
        .all(|position| matrix.get(position) == Some(MinoKind::E))
}

/// Lock the given piece into the matrix, at the position and rotation it comes with.
fn lock_piece_at(matrix: &mut Matrix, mino: Mino, shape_table: &ShapeTable) {
    let old_board = matrix.data.clone();

    for &p in &shape_table.0[&ShapeParameters::from(&mino)] {
        *(matrix.get_mut(p + mino.position).unwrap()) = mino.kind;
    }

    let mut real_ix = 0;
    for _ in 0..matrix.data.len() {
        if matrix.data[real_ix].iter().all(|&e| e != MinoKind::E) {
            matrix.data[real_ix..].rotate_left(1);
            matrix.data.last_mut().unwrap().fill(MinoKind::E);
        } else {
            real_ix += 1;
        }
    }

    // register updates made to the board
    let row_size = old_board[0].len();
    let new_updates = (0..).scan(
        (
            -1i32,
            old_board.into_iter().flat_map(|i| i.into_iter()),
            matrix.data.iter().flat_map(|i| i.iter()),
        ),
        |(offset, old, new), _| {
            itertools::diff_with(old.clone(), new.clone(), |a, b| a == *b).map(|d| match d {
                itertools::Diff::FirstMismatch(p, old_next, new_next) => {
                    *offset += p as i32 + 1;
                    *old = old_next.into_parts().1.clone();
                    let (Some(&kind), new_new) = new_next.into_parts() else {
                        unreachable!()
                    };
                    *new = new_new;
                    let loc = ivec2(*offset % row_size as i32, *offset / row_size as i32);
                    MatrixUpdate { loc, kind }
                }
                _ => unreachable!(),
            })
        },
    );

    matrix.updates.extend(new_updates);
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

const SOFT_DROP_SIZE: f32 = 1.5; // TODO this should be a multiplier on gravity, and not a constant
const SHIFT_SIZE: i32 = 1;

/// Update the state of the memory-representation of the board using player input
fn update_board(
    mut boards: Query<BoardQuery>,
    controller: Res<Controller>,
    shape_table: QueryShapeTable,
    kick_table: QueryKickTable,
) {
    for mut board in boards.iter_mut() {
        if let Some(mut p) = board.active.deref().0 {
            let farthest_legal_drop = (1..)
                .map(|o| (o, p.tap_mut(|p| p.position.y -= o)))
                .find(|(_, mino)| !has_free_space(&board.matrix, *mino, &shape_table))
                .map(|(o, _)| o - 1)
                .unwrap();

            if controller.hard_drop {
                // TODO when passive effects are added, this needs to happen when the piece locks
                // (by gravity or otherwise), not just during hard drop
                board.active.0.take();
                board.drop_clock.0 = 0.0;
                board.hold.activate();

                p.position.y -= farthest_legal_drop;
                lock_piece_at(&mut board.matrix, p, &shape_table);
            } else if controller.soft_drop {
                board.drop_clock.0 += SOFT_DROP_SIZE;
            }
        }

        if board.active.deref().0.is_none() {
            // TODO confirm that the piece can spawn before spawning it
            board.active.0 = Some(Mino {
                kind: board.queue.take(),
                position: ivec2(4, 22) - TEXTURE_CENTER_OFFSET,
                rotation: RotationState::Up,
            });
        }

        let mino = board.active.deref().0.unwrap();
        let farthest_legal_drop = (1..)
            .map(|o| (o, mino.tap_mut(|p| p.position.y -= o)))
            .find(|(_, mino)| !has_free_space(&board.matrix, *mino, &shape_table))
            .map(|(o, _)| o - 1)
            .unwrap();

        let old_drop_clock = board.drop_clock.deref().0;
        // The drop clock should only either drop the piece or lock it, NOT BOTH. This is so
        // that the player has time to interact with the piece when it hits the bottom, for a
        // frame at the very least. Later, we may want to rethink this for zero lock delay, if
        // such a thing makes sense.
        if farthest_legal_drop == 0 {
            // TODO lock delay
        } else if old_drop_clock > 1.0 {
            board.drop_clock.0 = old_drop_clock.fract();
            let drop_distance = std::cmp::min(old_drop_clock.trunc() as i32, farthest_legal_drop);
            board.active.0.as_mut().unwrap().position.y -= drop_distance;
        }

        let mino = board.active.0.unwrap();
        let original_rotation = mino.rotation;
        let rotation = if controller.rotate_180 {
            Some(original_rotation.rotate_180())
        } else if controller.rotate_left {
            Some(original_rotation.rotate_left())
        } else if controller.rotate_right {
            Some(original_rotation.rotate_right())
        } else {
            None
        };

        if let Some(new_rotation) = rotation {
            let kick_params = KickParameters {
                kind: mino.kind,
                from: original_rotation,
                to: new_rotation,
            };
            let kicks = kick_table.0.get(&kick_params);
            let offsets =
                std::iter::once(ivec2(0, 0)).chain(kicks.iter().flat_map(|p| p.iter()).copied());

            let successful_rot = offsets
                .map(|o| {
                    mino.tap_mut(|m| {
                        m.rotation = new_rotation;
                        m.position += o;
                    })
                })
                .find(|m| has_free_space(board.matrix.deref(), *m, &shape_table));

            if let Some(successful_rot) = successful_rot {
                board.active.0 = Some(successful_rot);
            }
        }

        let mino = board.active.deref().0.unwrap();
        let farthest_shift_left = (1..)
            .map(|o| (o, mino.tap_mut(|p| p.position.x -= o)))
            .find(|(_, mino)| !has_free_space(&board.matrix, *mino, &shape_table))
            .map(|(o, _)| o - 1)
            .unwrap();
        let farthest_shift_right = (1..)
            .map(|o| (o, mino.tap_mut(|p| p.position.x += o)))
            .find(|(_, mino)| !has_free_space(&board.matrix, *mino, &shape_table))
            .map(|(o, _)| o - 1)
            .unwrap();
        let shift_size = if controller.shift_left {
            -std::cmp::min(1, farthest_shift_left)
        } else if controller.shift_right {
            std::cmp::min(1, farthest_shift_right)
        } else if controller.repeat_left {
            -std::cmp::min(SHIFT_SIZE, farthest_shift_left)
        } else if controller.repeat_right {
            std::cmp::min(SHIFT_SIZE, farthest_shift_right)
        } else {
            0
        };
        if shift_size != 0 {
            board.active.0.as_mut().unwrap().position.x += shift_size;
        }

        if controller.hold {
            if let &Hold::Active(p) = board.hold.deref() {
                *(board.hold) = Hold::Inactive(board.active.0.unwrap().kind);
                board.active.0 = Some(Mino {
                    kind: p,
                    position: ivec2(4, 22) - TEXTURE_CENTER_OFFSET,
                    rotation: RotationState::Up,
                });
            } else if matches!(board.hold.deref(), Hold::Empty) {
                *(board.hold) = Hold::Inactive(board.active.0.unwrap().kind);
                board.active.0 = Some(Mino {
                    kind: board.queue.take(),
                    position: ivec2(4, 22) - TEXTURE_CENTER_OFFSET,
                    rotation: RotationState::Up,
                })
            }
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
                    set_camera_scale,
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
