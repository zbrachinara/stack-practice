use bevy::{
    math::vec2,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};

use crate::{
    assets::{
        tables::{shape_table::ShapeParameters, QueryShapeTable},
        MinoTextures,
    },
    board::{Active, Bounds, MinoKind, CELL_SIZE},
    image_tools::stack_images,
};

use super::matrix::MatrixMaterial;

#[derive(Component)]
pub struct ActiveSprite;

pub(super) fn spawn_active_sprite(
    mut commands: Commands,
    boards: Query<Entity, Added<Active>>,
    mino_textures: Res<MinoTextures>,
    shape_table: QueryShapeTable,
    mut images: ResMut<Assets<Image>>,
    mut mesh_server: ResMut<Assets<Mesh>>,
    mut material_server: ResMut<Assets<MatrixMaterial>>,
) {
    for e in boards.iter() {
        let dimensions = (shape_table.bounds[1] - shape_table.bounds[0]).as_uvec2();

        let all_textures = stack_images(&mino_textures.view(), &images);
        let material = MatrixMaterial {
            dimensions,
            mino_textures: images.add(all_textures),
            data: vec![0; (dimensions.x * dimensions.y) as usize],
        };

        let lo_f32 = shape_table.bounds[0].as_vec2();
        let hi_f32 = (shape_table.bounds[1]).as_vec2();
        let mesh = Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                [
                    lo_f32,
                    vec2(lo_f32.x, hi_f32.y),
                    hi_f32,
                    vec2(hi_f32.x, lo_f32.y),
                ]
                .map(|i| i.extend(0.) * (CELL_SIZE as f32))
                .to_vec(),
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_UV_0,
                vec![[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            )
            .with_indices(Some(Indices::U32(vec![0, 3, 1, 1, 3, 2])));

        let active_sprite = commands
            .spawn(MaterialMesh2dBundle {
                material: material_server.add(material),
                mesh: mesh_server.add(mesh).into(),
                ..default()
            })
            .insert(ActiveSprite)
            .id();

        commands.entity(e).add_child(active_sprite);
    }
}

/// Updates the visual state of the active piece. The active piece is a child of the board,
/// initialized in the same system that spawns the board. If the active pice becomes `None`, then
/// the sprite representing it is hidden. If it is modified in any other way, the sprite's position
/// and kind will be updated to match.
pub(super) fn display_active(
    active: Query<(&Active, &Bounds, &Children), Changed<Active>>,
    mut sprites: Query<
        (&mut Visibility, &mut Transform, &Handle<MatrixMaterial>),
        With<ActiveSprite>,
    >,
    shape_table: QueryShapeTable,
    mut material_server: ResMut<Assets<MatrixMaterial>>,
) {
    for (Active(e), bounds, children) in active.iter() {
        let active_sprite_id = children.iter().copied().find(|&c| sprites.contains(c));
        let (mut vis, mut pos, tex) = sprites.get_mut(active_sprite_id.unwrap()).unwrap();
        let mat = material_server.get_mut(tex).unwrap();

        if let Some(piece) = e {
            *vis = Visibility::Inherited;

            let offset = -(bounds.legal_bounds.as_vec2() / 2.);
            let new_pos = (piece.position.as_vec2() + offset) * CELL_SIZE as f32;
            pos.translation = new_pos.extend(1.0);

            mat.data.fill(MinoKind::E as u32);
            let shape = &shape_table.table[&ShapeParameters::from(piece)];
            for &p in shape {
                let loc = p - shape_table.bounds[0];
                let ix = loc.y * ((shape_table.bounds[1] - shape_table.bounds[0]).x) + loc.x;
                mat.data[ix as usize] = piece.kind as u32;
            }
        } else {
            *vis = Visibility::Hidden
        }
    }
}
