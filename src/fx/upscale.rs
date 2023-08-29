use super::post_process::PostProcessResources;
use super::PostProcessState;
use crate::traits::*;
use crate::{model::GfxState, traits::PostProcessFx};
use egui_wgpu::wgpu;

pub struct Upscale {
    res: UpscaleResources,
    pipeline: wgpu::ComputePipeline,
    fx_bind_group_layout: wgpu::BindGroupLayout,
    pub out_bind_group_layout: wgpu::BindGroupLayout,
}

impl PostProcessFx for Upscale {
    fn resize(&mut self, gfx_state: &GfxState, dispatch_xy: &[u32; 2]) {
        self.res = UpscaleResources::new(
            gfx_state,
            &self.fx_bind_group_layout,
            &self.out_bind_group_layout,
            dispatch_xy,
        );
    }

    fn compute<'a>(&'a self, input: Vec<&'a wgpu::BindGroup>, c_pass: &mut wgpu::ComputePass<'a>) {
        let UpscaleResources {
            fx_bind_group: bind_group,
            count_x,
            count_y,
            ..
        } = &self.res;

        c_pass.set_pipeline(&self.pipeline);
        c_pass.set_bind_group(0, input[0], &[]);
        c_pass.set_bind_group(1, &bind_group, &[]);
        c_pass.dispatch_workgroups(*count_x, *count_y, 1);
    }

    fn enabled(&self) -> bool {
        true
    }
}

impl Upscale {
    pub fn output(&self) -> &wgpu::BindGroup {
        &self.res.out_bind_group
    }

    pub fn new(
        gfx_state: &GfxState,
        in_bind_group_layout: &wgpu::BindGroupLayout,
        pp_res: &PostProcessResources,
    ) -> Self {
        let device = &gfx_state.device;

        let smoothen_shader = device.create_shader("fx/upscale.wgsl", "Upscale");

        let fx_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Upscale in textures layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: PostProcessState::TEXTURE_FORMAT,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                }],
            });

        let out_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Upscale out textures layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Upscale"),
            bind_group_layouts: &[in_bind_group_layout, &fx_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Upscale pipeline"),
            layout: Some(&pipeline_layout),
            module: &smoothen_shader,
            entry_point: "main",
        });

        let res = UpscaleResources::new(
            gfx_state,
            &fx_bind_group_layout,
            &out_bind_group_layout,
            &pp_res.dispatch_xy(),
        );

        Self {
            fx_bind_group_layout,
            out_bind_group_layout,
            pipeline,
            res,
        }
    }
}

struct UpscaleResources {
    fx_bind_group: wgpu::BindGroup,
    out_bind_group: wgpu::BindGroup,
    count_x: u32,
    count_y: u32,
}

impl UpscaleResources {
    pub fn new(
        gfx_state: &GfxState,
        fx_bind_group_layout: &wgpu::BindGroupLayout,
        out_bind_group_layout: &wgpu::BindGroupLayout,
        dispatch_xy: &[u32; 2],
    ) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let fx_view = gfx_state.create_fx_view(config.width, config.height);

        let fx_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &fx_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&fx_view),
            }],
        });

        let out_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &out_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&fx_view),
            }],
        });

        Self {
            fx_bind_group,
            out_bind_group,
            count_x: dispatch_xy[0],
            count_y: dispatch_xy[1],
        }
    }
}
