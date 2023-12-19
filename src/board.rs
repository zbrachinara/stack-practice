use std::time::Duration;

use bevy::{
    app::{Plugin, PostUpdate, Startup, Update},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, Input},
    math::IVec2,
    time::Time,
    transform::components::Transform,
    utils::HashMap,
};

#[derive(Default, Component)]
struct Matrix {
    grid: HashMap<IVec2, Entity>,
}

#[derive(Bundle, Default)]
pub struct Board {
    transform: Transform,
    matrix: Matrix,
}

#[derive(Resource, Default)]
pub struct Controller {
    shift_left: bool,
    shift_right: bool,
    repeat_left: bool,
    repeat_right: bool,
    repeater_left: Repeatable,
    repeater_right: Repeatable,

    hard_drop: bool,
    soft_drop: bool,

    rotate_left: bool,
    rotate_right: bool,
    rotate_180: bool,

    hold: bool,
}

const REPEAT_START_DELAY: Duration = Duration::from_millis(2000);
const REPEAT_DELAY: Duration = Duration::from_millis(100);

#[derive(Default, Clone, Copy)]
struct Repeatable {
    repeat_at: Option<Duration>,
}

impl Repeatable {
    fn update(&mut self, time: &Res<Time>, activation: bool) -> (bool, bool) {
        if activation {
            if let Some(time_to_repeat) = self.repeat_at {
                if time_to_repeat < time.elapsed() {
                    tracing::debug!("registered a repeat activation");
                    let now = time.elapsed();
                    // self.repeat_activation = true;
                    self.repeat_at = Some(now + REPEAT_DELAY);
                    return (false, true);
                }
            } else {
                // key has been pressed for the first time
                tracing::debug!("registered a single activation");
                let now = time.elapsed();
                self.repeat_at = Some(now + REPEAT_START_DELAY);
                return (true, false);
            }
        } else {
            // key was released, deactivate repeats
            self.repeat_at = None;
        }

        (false, false)
    }
}

/// Turns raw kb input into controller input which directly maps to actions on the board
fn process_input(keys: Res<Input<KeyCode>>, time: Res<Time>, mut controller: ResMut<Controller>) {
    tracing::debug_span!(module_path!());

    if keys.just_pressed(KeyCode::Space) {
        controller.hard_drop = true;
    }
    if keys.just_pressed(KeyCode::S) {
        controller.soft_drop = true;
    }
    if keys.just_pressed(KeyCode::Comma) {
        controller.rotate_left = true;
    }
    if keys.just_pressed(KeyCode::Slash) {
        controller.rotate_right = true;
    }
    if keys.just_pressed(KeyCode::Period) {
        controller.rotate_180 = true;
    }
    if keys.just_pressed(KeyCode::Tab) {
        controller.hold = true;
    }

    // repeatable keys
    let (shift_left, repeat_left) = controller
        .repeater_left
        .update(&time, keys.pressed(KeyCode::A));
    let (shift_right, repeat_right) = controller
        .repeater_right
        .update(&time, keys.pressed(KeyCode::D));

    controller.shift_left = shift_left;
    controller.shift_right = shift_right;
    controller.repeat_left = repeat_left;
    controller.repeat_right = repeat_right;
}

fn reset_controller(mut controller: ResMut<Controller>) {
    let repeater_left = controller.repeater_left;
    let repeater_right = controller.repeater_right;
    std::mem::take(&mut *controller);
    controller.repeater_right = repeater_right;
    controller.repeater_left = repeater_left;
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
