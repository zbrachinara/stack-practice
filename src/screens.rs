use std::num::{ParseFloatError, ParseIntError};

use bevy::prelude::*;
use bevy::utils::thiserror;
use bevy_egui::egui::{Key, TextEdit};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use duplicate::duplicate;
use smart_default::SmartDefault;

use crate::{board::Settings, state::MainState};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<GlobalSettings>()
            .add_systems(Update, (settings_panel, apply_settings).chain())
            .add_systems(
                Update,
                start_playing
                    .run_if(in_state(MainState::Ready))
                    .after(apply_settings),
            )
            .add_systems(OnExit(MainState::Loading), setup_scene);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource, SmartDefault)]
pub struct GlobalSettings {
    #[default = "10"]
    pub soft_drop_power: String,
    #[default = "0.02"]
    pub gravity_power: String,
    #[default = "0.5"]
    pub lock_delay: String,
    #[default = "1000"]
    pub initial_delay: String,
    #[default = "100"]
    pub repeat_delay: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseNumError {
    #[error("Invalid float in settings: {0}")]
    Float(#[from] ParseFloatError),
    #[error("Invalid int in settings: {0}")]
    Int(#[from] ParseIntError),
}

impl TryFrom<&GlobalSettings> for Settings {
    type Error = ParseNumError;

    fn try_from(value: &GlobalSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            soft_drop_power: value.soft_drop_power.parse()?,
            gravity_power: value.gravity_power.parse()?,
            lock_delay: value.lock_delay.parse()?,
            initial_delay: value.initial_delay.parse()?,
            repeat_delay: value.repeat_delay.parse()?,
        })
    }
}

fn settings_panel(mut contexts: EguiContexts, mut settings: ResMut<GlobalSettings>) {
    egui::SidePanel::left("settings_panel").show(contexts.ctx_mut(), |ui| {
        let had_focus = ui.memory(|e| e.focus().is_some());
        let tab_pressed = ui.input(|i| i.key_pressed(Key::Tab));
        let must_surrender = !had_focus && tab_pressed;

        egui::Grid::new("settings_panel_inner").show(ui, |ui| {
            duplicate! {
                [
                    field               display_name;
                    [soft_drop_power]   ["Soft Drop Power"];
                    [gravity_power]     ["Gravity power"];
                    [lock_delay]        ["Lock Delay"];
                    [initial_delay]     ["Initial Delay"];
                    [repeat_delay]      ["Repeat Delay"]
                ]
                let mut copy = settings.field.clone();
                ui.label(display_name);

                let text_edit = ui.add(TextEdit::singleline(&mut copy));
                if must_surrender {
                    text_edit.surrender_focus();
                }

                if settings.field != copy {
                    settings.field = copy;
                }
                ui.end_row();
            }
        })
    });
}

pub fn apply_settings(
    global_settings: Res<GlobalSettings>,
    mut all_settings: Query<&mut Settings>,
) {
    if_chain::if_chain! {
        if global_settings.is_changed();
        if let Ok(global) = Settings::try_from(&*global_settings);
        then {
            for mut s in all_settings.iter_mut() {
                *s = global.clone()
            }
        }
    }
}

pub fn start_playing(
    input: Res<Input<KeyCode>>,
    mut state: ResMut<NextState<MainState>>,
    settings: Res<GlobalSettings>,
) {
    if input.just_pressed(KeyCode::Grave) && Settings::try_from(&*settings).is_ok() {
        state.0 = Some(MainState::Playing);
    }
}
