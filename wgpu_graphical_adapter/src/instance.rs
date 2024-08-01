use wgpu::util::DeviceExt;

#[derive(Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn instances_to_buffer(instances: &[Instance], device: &wgpu::Device) -> wgpu::Buffer {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
         device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    } 
    
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}