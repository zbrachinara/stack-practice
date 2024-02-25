use bevy::ecs::query::QueryData;
use bevy::math::{ivec2, IVec2};
use bevy::prelude::*;
use smart_default::SmartDefault;

pub mod queue;
pub mod update;

use crate::controller::process_input;
use crate::replay::record::PreviousMatrix;
use crate::{screens::GlobalSettings, state::MainState};

use self::{
    queue::PieceQueue,
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
    pub fn rotate_180(self) -> Self {
        use RotationState::*;
        match self {
            Up => Down,
            Right => Left,
            Down => Up,
            Left => Right,
        }
    }

    pub fn rotate_left(self) -> Self {
        use RotationState::*;
        match self {
            Up => Left,
            Right => Up,
            Down => Right,
            Left => Down,
        }
    }

    pub fn rotate_right(self) -> Self {
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
pub struct Mino {
    pub kind: MinoKind,
    pub position: IVec2,
    pub rotation: RotationState,
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub enum Hold {
    #[default]
    Empty,
    Ready(MinoKind),
    Inactive(MinoKind),
}

impl Hold {
    pub fn activate(&mut self) {
        if let Self::Inactive(p) = self {
            *self = Self::Ready(*p);
        }
    }
}

pub const MATRIX_DEFAULT_SIZE: IVec2 = ivec2(10, 40);
pub const MATRIX_DEFAULT_LEGAL_BOUNDS: IVec2 = ivec2(10, 20);
pub const CELL_SIZE: u32 = 32;

#[derive(Component, SmartDefault)]
pub struct Bounds {
    #[default(MATRIX_DEFAULT_SIZE)]
    pub true_bounds: IVec2,
    #[default(MATRIX_DEFAULT_LEGAL_BOUNDS)]
    pub legal_bounds: IVec2,
}

#[derive(Component, Default)]
pub struct Active(pub Option<Mino>);

#[derive(Component)]
pub struct Matrix {
    pub data: Vec<Vec<MinoKind>>,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            data: std::iter::repeat_with(|| vec![MinoKind::E; MATRIX_DEFAULT_SIZE.x as usize])
                .take(MATRIX_DEFAULT_SIZE.y as usize)
                .collect(),
        }
    }
}

#[derive(Component, Default)]
pub struct DropClock {
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

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatrixAction {
    Insert,
    Erase,
}

#[derive(Debug, Clone, Copy)]
pub struct MatrixUpdate {
    pub loc: IVec2,
    pub old: MinoKind,
    pub new: MinoKind,
}

impl MatrixUpdate {
    pub fn invert(mut self) -> Self {
        std::mem::swap(&mut self.old, &mut self.new);
        self
    }
}

#[derive(Component, Clone, Debug)]
pub struct Settings {
    pub soft_drop_power: f32,
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
    previous_matrix: PreviousMatrix,
}

fn respawn_board(
    mut commands: Commands,
    old_boards: Query<Entity, With<Matrix>>,
    settings: Res<GlobalSettings>,
) {
    for e in old_boards.iter() {
        commands.entity(e).despawn_recursive();
    }
    commands.spawn(Board {
        settings: Settings::try_from(&*settings).unwrap(),
        ..default()
    });
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
                position: ivec2(4, 22),
                rotation: RotationState::Up,
            },
        });
    }
}

pub struct BoardPlugin;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct BoardQuery {
    pub matrix: &'static mut Matrix,
    pub active: &'static mut Active,
    pub hold: &'static mut Hold,
    pub queue: &'static mut PieceQueue,
    pub drop_clock: &'static mut DropClock,
    pub bounds: &'static Bounds,
    pub settings: &'static Settings,
    pub id: Entity,
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PieceSpawnEvent>()
            // .add_systems(Startup, register_start_game)
            .add_systems(OnEnter(MainState::Ready), respawn_board)
            .add_systems(
                OnTransition {
                    from: MainState::Ready,
                    to: MainState::Playing,
                },
                start_game,
            )
            .add_systems(
                Update,
                (
                    spawn_piece.run_if(on_event::<PieceSpawnEvent>()),
                    update_board.after(process_input),
                )
                    .run_if(in_state(MainState::Playing)),
            );
    }
}
