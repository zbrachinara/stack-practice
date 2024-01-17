#![allow(clippy::type_complexity)]

use bevy::{
    app::{Last, Plugin, PostUpdate, Startup, Update},
    asset::{AssetPath, Handle},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::With,
        schedule::{
            common_conditions::{in_state, on_event, resource_exists},
            IntoSystemConfigs, OnEnter,
        },
        system::{Commands, Query, Res, Resource, SystemId},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    math::{ivec2, IVec2},
    prelude::Deref,
    render::{
        color::Color,
        texture::Image,
        view::{InheritedVisibility, Visibility},
    },
    time::Time,
    transform::components::{GlobalTransform, Transform},
};
use smart_default::SmartDefault;
use std::time::Duration;
use tap::Tap;

mod controller;
mod display;
mod queue;
mod record;
mod replay;
mod update;

use crate::{
    assets::{tables::shape_table::ShapeParameters, MinoTextures},
    screens::GlobalSettings,
    state::MainState,
};

use self::{
    controller::{process_input, reset_controller, Controller},
    display::BoardDisplayPlugin,
    queue::PieceQueue,
    record::{discretized_time, record, FirstFrame, Record},
    replay::ReplayPlugin,
    update::{spawn_piece, update_board, PieceSpawnEvent},
};

#[derive(
    Debug, PartialEq, Eq, Hash, Clone, Copy,
    serde::Serialize, serde::Deserialize, strum::EnumIter
)]
#[repr(u32)]
#[rustfmt::skip]
pub enum MinoKind {
    E = 0, T, O, L, J, S, Z, I, G,
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

    pub fn color(&self) -> Color {
        match self {
            MinoKind::T => Color::PURPLE,
            MinoKind::O => Color::YELLOW,
            MinoKind::L => Color::ORANGE,
            MinoKind::J => Color::BLUE,
            MinoKind::S => Color::LIME_GREEN,
            MinoKind::Z => Color::RED,
            MinoKind::I => Color::AQUAMARINE,
            MinoKind::G => todo!(),
            MinoKind::E => todo!(),
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

#[derive(Component, Default, Clone, Copy, Debug)]
enum Hold {
    #[default]
    Empty,
    Ready(MinoKind),
    Inactive(MinoKind),
}

impl Hold {
    fn activate(&mut self) {
        if let Self::Inactive(p) = self {
            *self = Self::Ready(*p);
        }
    }
}

const MATRIX_DEFAULT_SIZE: IVec2 = ivec2(10, 40);
const MATRIX_DEFAULT_LEGAL_BOUNDS: IVec2 = ivec2(10, 20);
/// The amount by which the spawn location of the piece is offset from the bottom left corner of its
/// texture. This should be uniform for all pieces, hence why it is declared constant here.
const TEXTURE_CENTER_OFFSET: IVec2 = ivec2(1, 2);
pub const CELL_SIZE: u32 = 32;

#[derive(Component, SmartDefault)]
struct Bounds {
    #[default(MATRIX_DEFAULT_SIZE)]
    true_bounds: IVec2,
    #[default(MATRIX_DEFAULT_LEGAL_BOUNDS)]
    legal_bounds: IVec2,
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

#[derive(Debug, Clone, Copy)]
struct MatrixUpdate {
    loc: IVec2,
    kind: MinoKind,
}

#[derive(Component, Clone, Debug)]
pub struct Settings {
    pub soft_drop_power: f32,
    pub shift_size: i32,
    pub gravity_power: f32,
    pub lock_delay: f32,
    pub initial_delay: u32,
    pub repeat_delay: u32,
}

impl Default for Settings {
    fn default() -> Self {
        (&GlobalSettings::default()).try_into().unwrap()
    }
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
    settings: Settings,
}

fn begin_game(
    mut commands: Commands,
    old_boards: Query<Entity, With<Matrix>>,
    start_game: Res<StartGame>,
    time: Res<Time>,
    settings: Res<GlobalSettings>,
) {
    for e in old_boards.iter() {
        commands.entity(e).despawn_recursive();
    }

    commands
        .spawn(Board::default().tap_mut(|b| b.settings = Settings::try_from(&*settings).unwrap()));
    commands.insert_resource(FirstFrame(discretized_time(&time)));
    commands.run_system(**start_game);
}

fn clear_update_queue(mut boards: Query<&mut Matrix>) {
    for mut board in boards.iter_mut() {
        board.updates.clear();
    }
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

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<Controller>()
            .init_resource::<Record>()
            .add_plugins((BoardDisplayPlugin, ReplayPlugin))
            .add_event::<PieceSpawnEvent>()
            .add_systems(Startup, register_start_game)
            .add_systems(OnEnter(MainState::Playing), begin_game)
            .add_systems(
                Update,
                (
                    process_input,
                    spawn_piece.run_if(on_event::<PieceSpawnEvent>()),
                    update_board.after(process_input),
                )
                    .run_if(in_state(MainState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (
                    reset_controller,
                    record.run_if(resource_exists::<FirstFrame>()),
                )
                    .run_if(in_state(MainState::Playing)),
            )
            .add_systems(Last, clear_update_queue);
    }
}
