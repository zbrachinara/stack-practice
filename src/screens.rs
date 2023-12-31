use bevy::{ecs::system::SystemId, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

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

#[derive(Resource, Default)]
pub struct GlobalSettings(pub Settings);

fn settings_panel(mut contexts: EguiContexts, mut settings: ResMut<GlobalSettings>) {
    egui::SidePanel::left("settings_panel").show(contexts.ctx_mut(), |ui| {
        egui::Grid::new("settings_pannel_inner").show(ui, |ui| {
            let mut soft_drop_power_str = settings.0.soft_drop_power.to_string();
            ui.label("Setting: Soft Drop Power");
            ui.text_edit_singleline(&mut soft_drop_power_str);

            println!("{soft_drop_power_str}");
            if let Ok(f) = soft_drop_power_str.parse::<f32>() {
                if settings.0.soft_drop_power != f {
                    settings.0.soft_drop_power = f;
                }
            }
            ui.end_row();
        })
    });
}

fn apply_settings(global_settings: Res<GlobalSettings>, mut all_settings: Query<&mut Settings>) {
    if global_settings.is_changed() {
        println!(
            "applying settings: sdf: {}",
            global_settings.0.soft_drop_power
        );
        for mut s in all_settings.iter_mut() {
            *s = global_settings.0.clone()
        }
    }
}
