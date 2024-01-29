use bevy::prelude::*;

pub const DEFAULT_CAMERA_ZOOM: f32 = 1.3;
pub const REPLAY_CAMERA_ZOOM: f32 = 1.5;

#[derive(Resource, Deref, DerefMut)]
pub struct CameraZoom(f32);

fn adjust_camera_zoom(zoom: Res<CameraZoom>, mut cameras: Query<&mut OrthographicProjection>) {
    let camera = cameras.single();
    let distance = camera.scale - **zoom;
    if distance.abs() > f32::EPSILON {
        let mut camera = cameras.single_mut();
        camera.scale -= distance / 10.0;
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraZoom(DEFAULT_CAMERA_ZOOM))
            .add_systems(
                Update,
                adjust_camera_zoom.run_if(|q: Query<&OrthographicProjection>| !q.is_empty()),
            );
    }
}
