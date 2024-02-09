use crate::assets::image_tools::stack_images;
use crate::assets::MinoTextures;
use crate::board::CELL_SIZE;
use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy::math::{ivec2, IRect, IVec2, UVec2};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle};
use tap::Pipe;

#[derive(Clone, TypePath, Asset, AsBindGroup)]
pub struct MatrixMaterial {
    #[uniform(0)]
    pub dimensions: UVec2,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub mino_textures: Handle<Image>,
    #[storage(3)]
    pub data: Vec<u32>,
}

impl Material2d for MatrixMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/matrix.wgsl".into()
    }
}

#[derive(SystemParam)]
pub struct MatrixMaterialSpawner<'w, 's> {
    commands: Commands<'w, 's>,
    texture_server: ResMut<'w, Assets<Image>>,
    material_server: ResMut<'w, Assets<MatrixMaterial>>,
    mesh_server: ResMut<'w, Assets<Mesh>>,
    mino_textures: Res<'w, MinoTextures>,
}

fn corners(r: IRect) -> [IVec2; 4] {
    [
        r.min,
        ivec2(r.min.x, r.max.y),
        r.max,
        ivec2(r.max.x, r.min.y),
    ]
}

impl<'w, 's> MatrixMaterialSpawner<'w, 's> {
    fn quad_anchored(&mut self, r: IRect) -> Mesh2dHandle {
        let mesh_struct = Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                corners(r)
                    .map(|i| i.as_vec2().extend(0.) * (CELL_SIZE as f32))
                    .to_vec(),
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_UV_0,
                vec![[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            )
            .with_indices(Some(Indices::U32(vec![0, 3, 1, 1, 3, 2])));

        self.mesh_server.add(mesh_struct).into()
    }

    pub fn spawn_centered(&mut self, bounds: IVec2) -> EntityCommands<'w, 's, '_> {
        self.spawn(IRect::from_center_size(IVec2::ZERO, bounds))
    }

    pub fn spawn_centered_with_data(
        &mut self,
        bounds: IVec2,
        data: Vec<u32>,
    ) -> EntityCommands<'w, 's, '_> {
        self.spawn_with_data(IRect::from_center_size(IVec2::ZERO, bounds), data)
    }

    pub fn spawn(&mut self, grid_bounds: IRect) -> EntityCommands<'w, 's, '_> {
        self.spawn_with_data(
            grid_bounds,
            vec![0; grid_bounds.size().pipe(|u| u.x * u.y) as usize],
        )
    }

    pub fn spawn_with_data(
        &mut self,
        grid_bounds: IRect,
        data: Vec<u32>,
    ) -> EntityCommands<'w, 's, '_> {
        let all_textures = stack_images(&self.mino_textures.view(), &self.texture_server);
        let size = grid_bounds.size();

        assert_eq!((size.x * size.y) as usize, data.len());

        let material = MatrixMaterial {
            dimensions: grid_bounds.size().as_uvec2(),
            mino_textures: self.texture_server.add(all_textures),
            data,
        };
        let mesh = self.quad_anchored(grid_bounds);

        self.commands.spawn(MaterialMesh2dBundle {
            material: self.material_server.add(material),
            mesh,
            ..default()
        })
    }
}
