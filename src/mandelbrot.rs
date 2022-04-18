use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            std140::{AsStd140, Std140},
            *,
        },
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPipeline, Material2dPlugin, MaterialMesh2dBundle},
};

#[derive(Debug, Clone, TypeUuid, Component)]
#[uuid = "d29793f4-c24d-43f0-97c7-4d417a99188a"]
pub struct MandelbrotMaterial {
    pub center: Vec2,
    pub start: Vec2,
    pub scale: f32,
    pub aspect: f32,
    pub iters: i32,
}

#[derive(Clone, Default, AsStd140)]
pub struct MandelbrotFSUniformData {
    pub center: Vec2,
    pub start: Vec2,
    pub scale: f32,
    pub aspect: f32,
    pub iters: i32,
}

#[derive(Debug, Clone)]
pub struct GpuMandelbrotMaterial {
    pub fs_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl RenderAsset for MandelbrotMaterial {
    type ExtractedAsset = MandelbrotMaterial;
    type PreparedAsset = GpuMandelbrotMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<Material2dPipeline<MandelbrotMaterial>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, mandelbrot_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let fs_value = MandelbrotFSUniformData {
            center: material.center,
            start: material.start,
            scale: material.scale,
            aspect: material.aspect,
            iters: material.iters,
        };
        let fs_value_std140 = fs_value.as_std140();

        let fs_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("mandelbrot_material_uniform_fs_buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: fs_value_std140.as_bytes(),
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: fs_buffer.as_entire_binding(),
            }],
            label: Some("mandelbrot_material_bind_group"),
            layout: &mandelbrot_pipeline.material2d_layout,
        });

        Ok(GpuMandelbrotMaterial {
            fs_buffer,
            bind_group,
        })
    }
}

impl Material2d for MandelbrotMaterial {
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/mandelbrot.wgsl"))
    }

    #[inline]
    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(
                        MandelbrotFSUniformData::std140_size_static() as u64,
                    ),
                },
                count: None,
            }],
            label: Some("mandelbrot_material_layout"),
        })
    }
}

pub type MandelbrotMesh2dBundle = MaterialMesh2dBundle<MandelbrotMaterial>;
pub type MandelbrotPlugin = Material2dPlugin<MandelbrotMaterial>;
