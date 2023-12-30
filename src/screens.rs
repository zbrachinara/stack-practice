use bevy::{ecs::system::SystemId, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::state::MainState;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, |w: &mut World| {
                let id = w.register_system(set_camera_scale);
                w.insert_resource(SetCameraScale(id));
            })
            .add_systems(Update, test)
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

fn test(mut contexts: EguiContexts) {
    egui::SidePanel::left("settings_panel").show(contexts.ctx_mut(), |ui| ui.label("test"));
}
