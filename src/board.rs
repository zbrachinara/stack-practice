use bevy::{
    app::{Plugin, Update},
    ecs::{
        entity::Entity,
        schedule::IntoSystemConfigs,
        system::{Commands, Res, ResMut, Resource},
    },
    input::{keyboard::ScanCode, Input},
    math::{IVec2, Vec3},
    transform::components::Transform,
    utils::{default, HashMap},
};

#[derive(Resource, Default)]
pub struct Board {
    transform: Transform,
    grid: HashMap<IVec2, Entity>,
}

#[derive(Resource)]
pub struct Controller {}

/// Turns raw kb input into controller input which directly maps to actions on the board
fn process_input(keys: Res<Input<ScanCode>>, controller: ResMut<Controller>) {
    unimplemented!()
}

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
fn redraw_board(mut commands: Commands, board: Res<Board>, controller: Res<Controller>) {
    unimplemented!()
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Board {
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        })
        .insert_resource(Controller {})
        .add_systems(Update, (process_input, redraw_board.after(process_input)));
    }
}
