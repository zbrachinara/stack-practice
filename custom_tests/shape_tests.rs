use bevy::{app::App, DefaultPlugins};
use quickstacking::assets::MinoPlugin;

fn render_all_pieces() {

}

fn main() {
    App::new().add_plugins((DefaultPlugins, MinoPlugin));
}
