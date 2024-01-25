use crate::board::Settings;
use crate::screens::GlobalSettings;
use bevy::prelude::{DetectChanges, Local};
use bevy::{
    ecs::system::{Res, ResMut, Resource},
    input::{keyboard::KeyCode, Input},
    time::Time,
};

#[rustfmt::skip]
#[derive(Copy, Clone)]
pub enum RotateCommand {
    Left, Right, R180,
}

#[derive(Resource, Default)]
pub struct Controller {
    // pub shift_left: u32,
    // pub shift_right: u32,
    pub shift: i32,
    repeater_left: Repeatable,
    repeater_right: Repeatable,

    pub hard_drop: bool,
    pub soft_drop: bool,

    /// Signals that the active piece should rotate the piece to the left. The meaning of "rotate"
    /// here is that, if the piece is embedded in a wheel (like a driving wheel), the wheel is
    /// rotated to the left, and the piece along with it. How exactly the piece is "embedded in that
    /// wheel", so to speak, is encoded by the shape table.
    pub rotation: Option<RotateCommand>,

    pub hold: bool,
}

#[derive(Clone, Copy, Default)]
struct Repeatable {
    /// Used to determine which repeater activated first.
    activated_at: f32,
    repeat_at: Option<u32>,
}

impl Repeatable {
    fn initial_delay(&self, settings: &Settings) -> u32 {
        if settings.initial_delay == 0 {
            settings.repeat_delay
        } else {
            settings.initial_delay
        }
    }

    /// Each time this is called, returns the number of activations that should be registered.
    fn update(&mut self, time: &Res<Time>, settings: &Settings, activation: bool) -> u32 {
        if activation {
            if let Some(time_to_repeat) = self.repeat_at {
                let delta = time.delta().as_millis() as u32;
                self.repeat_at = Some(delta.abs_diff(time_to_repeat) % settings.repeat_delay);
                if time_to_repeat < delta {
                    tracing::debug!("registered a repeat activation");
                    let activations = (delta - time_to_repeat) / settings.repeat_delay + 1;
                    return activations;
                }
            } else {
                // key has been pressed for the first time
                tracing::debug!("registered a single activation");
                self.repeat_at = Some(self.initial_delay(settings));
                self.activated_at = time.elapsed_seconds_wrapped();
                return 1;
            }
        } else {
            // key was released, deactivate repeats
            self.repeat_at = None;
        }

        0
    }
}

/// Turns raw kb input into controller input which directly maps to actions on the board
pub fn process_input(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    settings: Res<GlobalSettings>,
    mut cached_settings: Local<Settings>,
    mut controller: ResMut<Controller>,
) {
    tracing::debug_span!(module_path!());

    if keys.just_pressed(KeyCode::Space) {
        controller.hard_drop = true;
    }
    if keys.pressed(KeyCode::S) {
        controller.soft_drop = true;
    }
    if keys.just_pressed(KeyCode::Comma) {
        controller.rotation = Some(RotateCommand::Left);
    }
    if keys.just_pressed(KeyCode::Slash) {
        controller.rotation = Some(RotateCommand::Right);
    }
    if keys.just_pressed(KeyCode::Period) {
        controller.rotation = Some(RotateCommand::R180);
    }
    if keys.just_pressed(KeyCode::Tab) {
        controller.hold = true;
    }

    if_chain::if_chain! {
        if settings.is_changed();
        if let Ok(global) = Settings::try_from(&*settings);
        then {
            *cached_settings = global;
        }
    }

    // repeatable keys
    let shift_left =
        -(controller
            .repeater_left
            .update(&time, &cached_settings, keys.pressed(KeyCode::A)) as i32);
    let shift_right =
        controller
            .repeater_right
            .update(&time, &cached_settings, keys.pressed(KeyCode::D)) as i32;

    // if both left and right shift is active, take the one activated latest, or, if they were activated around the same
    // time, prefer left.
    if controller.repeater_left.repeat_at.is_some() && controller.repeater_right.repeat_at.is_some()
    {
        if controller.repeater_left.activated_at < controller.repeater_right.activated_at {
            controller.shift = shift_right;
        } else {
            controller.shift = shift_left;
        }
    } else {
        controller.shift = shift_left + shift_right;
    }
}

pub fn reset_controller(mut controller: ResMut<Controller>) {
    let repeater_left = controller.repeater_left;
    let repeater_right = controller.repeater_right;
    std::mem::take(&mut *controller);
    controller.repeater_right = repeater_right;
    controller.repeater_left = repeater_left;
}
