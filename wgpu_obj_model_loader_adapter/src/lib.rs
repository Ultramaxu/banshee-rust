use wgpu::util::DeviceExt;
use wgpu_graphical_adapter::gateways::WgpuModelLoaderGateway;
use wgpu_graphical_adapter::instance::Instance;
use wgpu_graphical_adapter::model::{Material, Mesh, Model};
use wgpu_graphical_adapter::texture::Texture;
use wgpu_graphical_adapter::vertex::ModelVertex;

pub struct ObjWgpuModelLoaderAdapter {
    out_dir: Box<str>
}

impl ObjWgpuModelLoaderAdapter {
    pub fn new(
        out_dir: Box<str>,
    ) -> Self {
        Self {
            out_dir
        }
    }

    fn load_binary_sync(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let data = std::fs::read(self.get_out_dir_path(file_name))?;
        Ok(data)
    }

    pub fn load_texture_sync(
        &self,
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<Texture> {
        let data = self.load_binary_sync(file_name)?;
        Texture::new_diffuse_texture_from_bytes(data, device, queue)
    }

    fn get_out_dir_path(&self, file_name: &str) -> std::path::PathBuf {
        std::path::Path::new(self.out_dir.as_ref())
            .join("res")
            .join(file_name)
    }
}

impl WgpuModelLoaderGateway for ObjWgpuModelLoaderAdapter {
    fn load_model_sync(
        &self,
        file_name: &str,
        instances: Vec<Instance>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        bind_group_builder: Box<dyn Fn(
            &wgpu::Device,
            &wgpu::TextureView,
            &wgpu::Sampler,
            &wgpu::BindGroupLayout
        ) -> wgpu::BindGroup>,
    ) -> anyhow::Result<Model> {
        let (models, obj_materials) = tobj::load_obj(
            self.get_out_dir_path(file_name),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        let mut materials = Vec::new();
        for m in obj_materials? {
            if let Some(diffuse_texture) = &m.diffuse_texture {
                let diffuse_texture = self.load_texture_sync(&diffuse_texture, device, queue)?;
                let bind_group = bind_group_builder(
                    device,
                    &diffuse_texture.view,
                    &diffuse_texture.sampler,
                    texture_bind_group_layout
                );

                materials.push(Material {
                    name: m.name,
                    diffuse_texture,
                    bind_group,
                })
            }
        }

        let meshes = models
            .into_iter()
            .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                    .map(|i| {
                        let position = [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ];
                        let tex_coords = [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]];
                        
                        if m.mesh.normals.is_empty() {
                            ModelVertex {
                                position,
                                tex_coords,
                                normal: [0.0, 0.0, 0.0],
                            }
                        } else {
                            ModelVertex {
                                position,
                                tex_coords,
                                normal: [
                                    m.mesh.normals[i * 3],
                                    m.mesh.normals[i * 3 + 1],
                                    m.mesh.normals[i * 3 + 2],
                                ],
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&m.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: m.mesh.indices.len() as u32,
                    material: m.mesh.material_id.unwrap_or(0),
                }
            })
            .collect::<Vec<_>>();


        let instances_buffer = Instance::instances_to_buffer(&instances, device);

        Ok(Model { 
            meshes,
            materials,
            instances: instances_buffer,
            num_instances: instances.len() as u32
        })
    }
}