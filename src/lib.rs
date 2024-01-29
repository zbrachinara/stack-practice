use bevy::app::PluginGroupBuilder;
use bevy::prelude::PluginGroup;

pub mod animation;
pub mod assets;
pub mod board;
pub mod screens;
pub mod state;

mod image_tools;

pub struct StackPracticePlugins;

impl PluginGroup for StackPracticePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(assets::StackingAssetsPlugin)
            .add(board::BoardPlugin)
            .add(state::StatePlugin)
            .add(screens::ScreensPlugin)
            .add(animation::AnimationPlugin)
    }
}
