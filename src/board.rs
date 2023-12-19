use bevy::{
    app::{Plugin, PostUpdate, Startup, Update},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res},
    },
    math::IVec2,
    transform::components::Transform,
    utils::HashMap,
};

mod controller;

use self::controller::{process_input, reset_controller, Controller};

#[derive(Default, Component)]
struct Matrix {
    grid: HashMap<IVec2, Entity>,
}

#[derive(Bundle, Default)]
pub struct Board {
    transform: Transform,
    matrix: Matrix,
}

fn spawn_board(mut commands: Commands) {
    commands.spawn(Board::default());
}

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
fn redraw_board(mut commands: Commands, board: Query<&mut Matrix>, controller: Res<Controller>) {
    // TODO complete
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Controller::default())
            .add_systems(Startup, spawn_board)
            .add_systems(Update, (process_input, redraw_board.after(process_input)))
            .add_systems(PostUpdate, reset_controller);
    }
}
