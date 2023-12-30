use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin).add_systems(Update, test);
    }
}

fn test(mut contexts: EguiContexts) {
    egui::SidePanel::left("settings_panel").show(contexts.ctx_mut(), |ui| ui.label("test"));
}
