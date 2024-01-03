use std::num::{ParseFloatError, ParseIntError};

use bevy::{ecs::system::SystemId, prelude::*, utils::thiserror};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use duplicate::duplicate;
use smart_default::SmartDefault;

use crate::{board::Settings, state::MainState};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<GlobalSettings>()
            .add_systems(Startup, |w: &mut World| {
                let id = w.register_system(set_camera_scale);
                w.insert_resource(SetCameraScale(id));
            })
            .add_systems(Update, (settings_panel, apply_settings).chain())
            .add_systems(OnExit(MainState::Loading), setup_scene);
    }
}

fn setup_scene(mut commands: Commands, scale_system: Res<SetCameraScale>) {
    commands.spawn(Camera2dBundle::default());
    commands.run_system(scale_system.0);
}

#[derive(Resource)]
struct SetCameraScale(SystemId);
fn set_camera_scale(mut camera: Query<&mut OrthographicProjection>) {
    camera.single_mut().scale = 2.0;
}

#[derive(Resource, SmartDefault)]
pub struct GlobalSettings {
    #[default = "10"]
    pub soft_drop_power: String,
    #[default = "1"]
    pub shift_size: String,
    #[default = "0.02"]
    pub gravity_power: String,
    #[default = "0.5"]
    pub lock_delay: String,
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
            shift_size: value.shift_size.parse()?,
            gravity_power: value.gravity_power.parse()?,
            lock_delay: value.lock_delay.parse()?,
        })
    }
}

fn settings_panel(mut contexts: EguiContexts, mut settings: ResMut<GlobalSettings>) {
    egui::SidePanel::left("settings_panel").show(contexts.ctx_mut(), |ui| {
        egui::Grid::new("settings_pannel_inner").show(ui, |ui| {
            duplicate! {
                [
                    field               display_name;
                    [soft_drop_power]   ["Soft Drop Power"];
                    [shift_size]        ["Shift Size"];
                    [gravity_power]     ["Gravity power"];
                    [lock_delay]        ["Lock Delay"];
                ]
                let mut copy = settings.field.clone();
                ui.label(display_name);
                ui.text_edit_singleline(&mut copy);
                if settings.field != copy {
                    settings.field = copy;
                }
                ui.end_row();
            }
        })
    });
}

pub fn apply_settings(global_settings: Res<GlobalSettings>, mut all_settings: Query<&mut Settings>) {
    if_chain::if_chain! {
        if global_settings.is_changed();
        if let Ok(global) = Settings::try_from(&*global_settings);
        then {
            for mut s in all_settings.iter_mut() {
                *s = global.clone()
            }
        }
    };
}
