use std::time::Duration;

use bevy::{
    app::{Plugin, PostUpdate, Update},
    ecs::{
        entity::Entity,
        schedule::IntoSystemConfigs,
        system::{Commands, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, Input},
    math::{IVec2, Vec3},
    time::Time,
    transform::components::Transform,
    utils::{default, HashMap},
};

#[derive(Resource, Default)]
pub struct Board {
    transform: Transform,
    grid: HashMap<IVec2, Entity>,
}

#[derive(Resource, Default)]
pub struct Controller {
    shift_left: bool,
    shift_right: bool,
    repeat_left: bool,
    repeat_right: bool,
    repeat_left_after: Option<Duration>,

    hard_drop: bool,
    soft_drop: bool,

    rotate_left: bool,
    rotate_right: bool,
    rotate_180: bool,

    hold: bool,
}

const REPEAT_START_DELAY: Duration = Duration::from_millis(2000);
const REPEAT_DELAY: Duration = Duration::from_millis(100);

struct Repeatable {
    
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
    if keys.pressed(KeyCode::A) {
        if let Some(time_to_repeat) = controller.repeat_left_after {
            if time_to_repeat < time.elapsed() {
                tracing::debug!("registered a left repeat");
                let now = time.elapsed();
                controller.repeat_left = true;
                controller.repeat_left_after = Some(now + REPEAT_DELAY);
            }
        } else {
            // key has been pressed for the first time
            tracing::debug!("registered a left shift");
            let now = time.elapsed();
            controller.shift_left = true;
            controller.repeat_left_after = Some(now + REPEAT_START_DELAY);
        }
    } else {
        // key was released, deactivate repeats
        tracing::debug!("left repeat inactive");
        controller.repeat_left_after = None;
    }
}

/// Creates/removes the tiles on the screen given the state of the board at the time. A variant of
/// each cell exists on the screen, and this system reads the currently active variant of tetromino
/// at that location and enables the visibility of that sprite accordingly.
fn redraw_board(mut commands: Commands, board: Res<Board>, controller: Res<Controller>) {
    // TODO complete
}

fn reset_controller(mut controller: ResMut<Controller>) {
    let repeat_left_after = controller.repeat_left_after;
    std::mem::take(&mut *controller);
    controller.repeat_left_after = repeat_left_after;
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Board {
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        })
        .insert_resource(Controller::default())
        .add_systems(Update, (process_input, redraw_board.after(process_input)))
        .add_systems(PostUpdate, reset_controller);
    }
}
