use bevy::{
    app::{App, PluginGroup},
    asset::AssetPlugin,
    utils::default,
    DefaultPlugins,
};

use quickstacking::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(false),
            ..default()
        }))
        .add_plugins(assets::StackingAssetsPlugin)
        .add_plugins(board::BoardPlugin)
        .add_plugins(state::StatePlugin)
        .run();
}
