use std::time::Duration;

use bevy::{
    ecs::system::{Res, ResMut, Resource},
    input::{keyboard::KeyCode, Input},
    time::Time,
};

#[derive(Resource, Default)]
pub struct Controller {
    pub shift_left: bool,
    pub shift_right: bool,
    pub repeat_left: bool,
    pub repeat_right: bool,
    repeater_left: Repeatable,
    repeater_right: Repeatable,

    pub hard_drop: bool,
    pub soft_drop: bool,

    /// Signals that the active piece should rotate the piece to the left. The meaning of "rotate"
    /// here is that, if the piece is embedded in a wheel (like a driving wheel), the wheel is
    /// rotated to the left, and the piece along with it. How exactly the piece is "embedded in that
    /// wheel", so to speak, is encoded by the shape table.
    pub rotate_left: bool,
    pub rotate_right: bool,
    pub rotate_180: bool,

    pub hold: bool,
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
pub fn process_input(keys: Res<Input<KeyCode>>, time: Res<Time>, mut controller: ResMut<Controller>) {
    tracing::debug_span!(module_path!());

    if keys.just_pressed(KeyCode::Space) {
        controller.hard_drop = true;
    }
    if keys.pressed(KeyCode::S) {
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

pub fn reset_controller(mut controller: ResMut<Controller>) {
    let repeater_left = controller.repeater_left;
    let repeater_right = controller.repeater_right;
    std::mem::take(&mut *controller);
    controller.repeater_right = repeater_right;
    controller.repeater_left = repeater_left;
}
