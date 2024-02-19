use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use smart_default::SmartDefault;

pub const PROGRESS_BAR_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(8714649747086695632918559878778085427);
pub struct ProgressBarPlugin;

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PROGRESS_BAR_HANDLE,
            "progress_shader.wgsl",
            Shader::from_wgsl
        );
        app.add_systems(Update, update_progress_bar)
            .add_plugins(UiMaterialPlugin::<ProgressBarMaterial>::default());
    }
}

#[repr(u8)]
#[derive(Default, Copy, Clone)]
#[allow(unused)]
pub enum Orientation {
    /// Horizontal, and progress bar moves toward the left
    Left = 0,
    /// Horizontal, and progress bar moves toward the right
    Right = 1,
    /// Vertical, and progress bar moves upward
    Up = 2,
    /// Vertical, and progress bar moves downward
    #[default]
    Down = 3,
}

/// The Progress Bar.
/// Has Different Colored section with relative size to each other
/// and a Color for the empty space
#[derive(Component, SmartDefault)]
pub struct ProgressBar {
    /// The Progress
    /// a f32 between 0.0 and 1.0
    pub progress: f32,
    /// The Different Sections
    /// The amount is the space relative to the other Sections.
    #[default(_code = "vec![(1, Color::WHITE)]")]
    pub sections: Vec<(u32, Color)>,
    /// The Color of the space that is not progressed to
    #[default(Color::NONE)]
    pub empty_color: Color,
    pub orientation: Orientation,
}

#[derive(Bundle)]
pub struct ProgressBarBundle {
    pub progressbar: ProgressBar,
    pub material_node_bundle: MaterialNodeBundle<ProgressBarMaterial>,
}

/// The Material for the ProgressBar
/// uses a simple wgsl shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
// #[uuid = "7d4aa28a-c01f-4ac7-b6f5-9b64cc3b4214"]
pub struct ProgressBarMaterial {
    #[uniform(0)]
    empty_color: Color,
    #[uniform(1)]
    progress: f32,
    /// The color of each section
    #[storage(2, read_only)]
    sections_color: Vec<Color>,
    #[storage(3, read_only)]
    sections_start_percentage: Vec<f32>,
    /// the length of the `sections_color` / `sections_start_percentage` vec.
    /// needs to be set for the shader
    #[uniform(4)]
    sections_count: u32,
    #[uniform(5)]
    orientation: u32,
}

impl From<&ProgressBar> for ProgressBarMaterial {
    fn from(bar: &ProgressBar) -> Self {
        let total_amount: u32 = bar.sections.iter().map(|(amount, _)| amount).sum();
        let (section_start_percentages, section_colors) = bar
            .sections
            .iter()
            .map(|(amount, color)| (*amount as f32 / total_amount as f32, *color))
            .unzip();

        Self {
            empty_color: bar.empty_color,
            progress: bar.progress,
            sections_count: bar.sections.len() as u32,
            sections_color: section_colors,
            sections_start_percentage: section_start_percentages,
            orientation: bar.orientation as u32,
        }
    }
}

impl Default for ProgressBarMaterial {
    fn default() -> Self {
        (&ProgressBar::default()).into()
    }
}

impl UiMaterial for ProgressBarMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        PROGRESS_BAR_HANDLE.into()
    }
}

fn update_progress_bar(
    bar_query: Query<(&ProgressBar, &Handle<ProgressBarMaterial>)>,
    mut materials: ResMut<Assets<ProgressBarMaterial>>,
) {
    for (bar, handle) in bar_query.iter() {
        if let Some(material) = materials.get_mut(handle) {
            *material = bar.into();
        }
    }
}
