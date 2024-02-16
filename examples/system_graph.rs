use bevy::app::{App, PostUpdate, PreUpdate, Update};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::DefaultPlugins;
use bevy_mod_debugdump::schedule_graph::Settings;
use bevy_mod_debugdump::schedule_graph_dot;
use stack_practice::StackPracticePlugins;
use std::io::Write;
use std::process::Stdio;
use std::thread::JoinHandle;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, StackPracticePlugins));

    [
        render_graph(&mut app, Update),
        render_graph(&mut app, PreUpdate),
        render_graph(&mut app, PostUpdate),
    ]
    .into_iter()
    .for_each(|j| j.join().unwrap())
}

fn render_graph(app: &mut App, schedule: impl ScheduleLabel) -> JoinHandle<()> {
    let settings = Settings::default().filter_in_crate("stack_practice");
    let name = format!("{}_systems", format!("{:?}", schedule).to_ascii_lowercase());

    let graph = schedule_graph_dot(app, schedule, &settings);
    let mut dot = std::process::Command::new("dot")
        .arg("-Tsvg")
        .arg(format!("-o{name}.svg"))
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    std::thread::spawn(move || {
        dot.stdin
            .take()
            .expect("Failed to open stdin")
            .write_all(graph.as_bytes())
            .expect("Could not write to stdin");

        dot.wait().unwrap();
    })
}
