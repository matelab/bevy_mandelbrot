use bevy::{
    core_pipeline::node::MAIN_PASS_DEPENDENCIES,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
        render_component::ExtractComponentPlugin,
        render_graph::{self, RenderGraph},
        render_resource::std140::{AsStd140, Std140},
        render_resource::*,
        renderer::{RenderContext, RenderDevice},
        RenderApp, RenderStage,
    },
};

pub struct ColormapPlugin {
    prev_node: &'static str,
}

impl ColormapPlugin {
    pub fn with_previous(prev_node: &'static str) -> Self {
        Self { prev_node }
    }
}

impl Plugin for ColormapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColormapInputImage>()
            .init_resource::<ColormapOutputImage>()
            .init_resource::<ColormapMappingImage>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ColormapPipeline>()
            .add_system_to_stage(RenderStage::Extract, extract_colormap)
            .add_system_to_stage(RenderStage::Queue, queue_bind_group);

        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        render_graph.add_node("colormap", ColormapDispatch);
        render_graph
            .add_node_edge("colormap", MAIN_PASS_DEPENDENCIES)
            .unwrap();

        render_graph
            .add_node_edge(self.prev_node, "colormap")
            .unwrap();
    }
}

#[derive(Default)]
pub struct ColormapInputImage(pub Handle<Image>);
#[derive(Default)]
pub struct ColormapOutputImage(pub Handle<Image>);
#[derive(Default)]
pub struct ColormapMappingImage(pub Handle<Image>);
struct ColormapBindGroup(BindGroup);

struct ColormapSize(Size);

struct ColormapPipeline {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}
struct ColormapDispatch;

impl FromWorld for ColormapPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let shader_source = include_str!("../assets/shaders/colormap.wgsl");
        let shader = render_device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("colormap_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let texture_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("colormap_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::R32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D1,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("colormap_pipline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("colormap_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "colormap",
        });

        ColormapPipeline {
            pipeline,
            bind_group_layout: texture_bind_group_layout,
        }
    }
}

impl render_graph::Node for ColormapDispatch {
    fn update(&mut self, _world: &mut World) {}

    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline = world.get_resource::<ColormapPipeline>().unwrap();
        if let Some(texture_bind_group) = world.get_resource::<ColormapBindGroup>() {
            let size = &world.get_resource::<ColormapSize>().unwrap();

            let mut pass = render_context
                .command_encoder
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_pipeline(&pipeline.pipeline);
            pass.set_bind_group(0, &texture_bind_group.0, &[]);
            pass.dispatch(
                (size.0.width / 8.0).ceil() as u32,
                (size.0.height / 8.0).ceil() as u32,
                1,
            );
        }

        Ok(())
    }
}

fn extract_colormap(
    mut commands: Commands,
    input: Res<ColormapInputImage>,
    output: Res<ColormapOutputImage>,
    mapping: Res<ColormapMappingImage>,
) {
    commands.insert_resource(ColormapInputImage(input.0.clone()));
    commands.insert_resource(ColormapOutputImage(output.0.clone()));
    commands.insert_resource(ColormapMappingImage(mapping.0.clone()));
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<ColormapPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    input: Res<ColormapInputImage>,
    output: Res<ColormapOutputImage>,
    mapping: Res<ColormapMappingImage>,
    render_device: Res<RenderDevice>,
) {
    if let (Some(input), Some(output), Some(mapping)) = (
        gpu_images.get(&input.0),
        gpu_images.get(&output.0),
        gpu_images.get(&mapping.0),
    ) {
        let ix = input.size.width.round() as i32;
        let iy = input.size.height.round() as i32;
        let ox = output.size.width.round() as i32;
        let oy = output.size.height.round() as i32;
        if (ix == ox) && (iy == oy) {
            let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                label: Some("colormap_bind_group"),
                layout: &pipeline.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&input.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&output.texture_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&mapping.texture_view),
                    },
                ],
            });
            commands.insert_resource(ColormapBindGroup(bind_group));
            commands.insert_resource(ColormapSize(input.size));
        }
    }
}
