use bevy::{app::App, DefaultPlugins};

use quickstacking::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(assets::MinoPlugin)
        .add_plugins(board::BoardPlugin)
        .run();
}
