use bevy::{app::App, DefaultPlugins};

mod board;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(board::BoardPlugin)
        .run();
}
