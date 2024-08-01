use crate::texture::Texture;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub instances: wgpu::Buffer,
    pub num_instances: u32,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub trait DrawModel<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instance_buffer: &'a wgpu::Buffer,
        instances: std::ops::Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        instances: Option<std::ops::Range<u32>>,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{    fn draw_mesh_instanced(
    &mut self,
    mesh: &'b Mesh,
    material: &'b Material,
    instance_buffer: &'b wgpu::Buffer,
    instances: std::ops::Range<u32>,
    camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_vertex_buffer(1, instance_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        instances: Option<std::ops::Range<u32>>,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(
                mesh,
                material, 
                &model.instances,
                instances.clone().unwrap_or(0..model.num_instances),
                camera_bind_group
            );
        }
    }
}