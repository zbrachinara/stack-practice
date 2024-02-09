use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
};
use bevy::sprite::{Material2d, MaterialMesh2dBundle};
use bevy::utils::HashSet;

use crate::assets::tables::QueryShapeTable;

use crate::board::{Active, Matrix, CELL_SIZE, MATRIX_DEFAULT_LEGAL_BOUNDS};

#[derive(Clone, TypePath, Asset, AsBindGroup)]
pub struct DropShadowMaterial {
    #[texture(1, dimension = "1d")]
    #[sampler(2)]
    base: Handle<Image>,
}

impl Material2d for DropShadowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/drop_shadow.wgsl".into()
    }
}

pub(crate) fn spawn_drop_shadow(
    mut commands: Commands,
    boards: Query<Entity, Added<Matrix>>,
    mut materials: ResMut<Assets<DropShadowMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    for b in boards.iter() {
        let image = images.add(Image::new_fill(
            Extent3d {
                width: 10,
                height: 1,
                ..default()
            },
            TextureDimension::D1,
            &[255, 255, 255, 255],
            TextureFormat::Rgba8UnormSrgb,
        ));

        let q = commands
            .spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(
                        shape::Quad::new(Vec2::new(
                            MATRIX_DEFAULT_LEGAL_BOUNDS.x as f32 * CELL_SIZE as f32,
                            256.,
                        ))
                        .into(),
                    )
                    .into(),
                material: materials.add(DropShadowMaterial {
                    base: image.clone(),
                }),
                transform: Transform::from_translation(
                    MATRIX_DEFAULT_LEGAL_BOUNDS.as_vec2().extend(0.0)
                        * vec3(0.0, -0.5, 0.0)
                        * (CELL_SIZE as f32)
                        - vec3(0.0, 256. / 2., 0.0),
                ),
                ..default()
            })
            .id();

        commands.entity(b).add_child(q);
    }
}

pub(crate) fn update_drop_shadow(
    active: Query<(&Active, &Children), Changed<Active>>,
    mat: Query<&Handle<DropShadowMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<DropShadowMaterial>>,
    shape_table: QueryShapeTable,
) {
    for (active, children) in active.iter() {
        if let Some(active) = active.0 {
            let child = children.iter().find_map(|e| mat.get(*e).ok()).unwrap();
            let material = mats.get_mut(child).unwrap();
            let image = images.get_mut(material.base.clone()).unwrap();

            let contained: HashSet<_> = shape_table[active]
                .iter()
                .map(|&p| (p + active.position).x as usize)
                .collect();

            for (i, chunk) in image.data.chunks_mut(4).enumerate() {
                let fill = if contained.contains(&i) {
                    active.kind.color()
                } else {
                    Color::WHITE
                };
                chunk.copy_from_slice(&fill.as_rgba_u8());
            }
        }
    }
}
