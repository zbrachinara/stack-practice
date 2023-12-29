use bevy::{
    app::{App, PluginGroup},
    asset::AssetPlugin,
    utils::default,
    DefaultPlugins,
};

use stack_practice::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(false),
                ..default()
            }),
            assets::StackingAssetsPlugin,
            board::BoardPlugin,
        ))
        .run();
}
