use bevy::prelude::*;
use stack_practice::StackPracticePlugins;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(false),
                ..default()
            }),
            StackPracticePlugins,
        ))
        .run();
}
