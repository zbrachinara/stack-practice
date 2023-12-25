#![allow(clippy::type_complexity)]

use bevy::{
    app::{Plugin, PostUpdate, Startup, Update},
    asset::{AssetPath, Assets, Handle},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter},
        system::{Commands, Query, Res, ResMut, Resource, SystemId},
        world::World,
    },
    hierarchy::BuildChildren,
    math::{ivec2, vec2, IVec2, UVec2},
    prelude::Deref,
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

mod controller;
mod display;
mod queue;
mod update;

use crate::{
    assets::{tables::shape_table::ShapeParameters, MinoTextures},
    state::MainState,
};

use self::{
    controller::{process_input, reset_controller, Controller},
    display::{center_board, display_active, display_held, display_queue, redraw_board},
    queue::PieceQueue,
    update::{spawn_piece, update_board, PieceSpawnEvent},
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Component, Default)]
struct DropClock {
    fall: f32,
    lock: f32,
}

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

#[derive(Bundle, Default)]
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
}

fn spawn_default_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn set_camera_scale(mut camera: Query<&mut OrthographicProjection>) {
    camera.single_mut().scale = 2.0;
}

fn spawn_board(
    mut commands: Commands,
    mut texture_server: ResMut<Assets<Image>>,
    start_game: Res<StartGame>,
) {
    let matrix_sprite = commands
        .spawn(SpriteBundle {
            texture: texture_server.add(transparent_texture(
                MATRIX_DEFAULT_SIZE.as_uvec2() * CELL_SIZE,
            )),
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

    let mut board = commands.spawn(Board::default());

    board
        .add_child(matrix_sprite)
        .add_child(active_sprite)
        .add_child(hold_sprite);
    for e in queue_sprites {
        board.add_child(e);
    }

    commands.run_system(**start_game);
}

#[derive(Resource, Deref)]
struct StartGame(SystemId);

fn register_start_game(w: &mut World) {
    let id = w.register_system(start_game);
    w.insert_resource(StartGame(id))
}

fn start_game(
    mut boards: Query<(Entity, &mut PieceQueue), With<Matrix>>,
    mut commands: EventWriter<PieceSpawnEvent>,
) {
    for (board, mut queue) in boards.iter_mut() {
        commands.send(PieceSpawnEvent {
            board,
            mino: Mino {
                kind: queue.take(),
                position: ivec2(4, 22) - TEXTURE_CENTER_OFFSET,
                rotation: RotationState::Up,
            },
        });
    }
}

impl From<&Mino> for ShapeParameters {
    fn from(&Mino { kind, rotation, .. }: &Mino) -> Self {
        ShapeParameters { kind, rotation }
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
            .add_event::<PieceSpawnEvent>()
            .add_systems(Startup, register_start_game)
            .add_systems(
                OnEnter(MainState::Playing),
                (spawn_board, spawn_default_camera),
            )
            .add_systems(
                Update,
                (
                    process_input,
                    spawn_piece,
                    update_board.after(process_input),
                )
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
