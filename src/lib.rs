use bevy::app::PluginGroupBuilder;
use bevy::prelude::PluginGroup;

pub mod animation;
pub mod assets;
pub mod board;
pub mod display;
pub mod replay;
pub mod screens;
pub mod state;

mod controller;
mod progress_bar;

pub struct StackPracticePlugins;

impl PluginGroup for StackPracticePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(progress_bar::ProgressBarPlugin)
            .add(assets::StackingAssetsPlugin)
            .add(controller::ControllerPlugin)
            .add(board::BoardPlugin)
            .add(display::DisplayPlugin)
            .add(replay::ReplayPlugin)
            .add(state::StatePlugin)
            .add(screens::ScreensPlugin)
            .add(animation::AnimationPlugin)
    }
}
