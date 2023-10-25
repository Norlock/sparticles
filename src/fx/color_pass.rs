#![allow(unused)]
use egui_wgpu::wgpu;

use super::{
    post_process::{FxIOUniform, PingPongState},
    ColorFxUniform, FxState,
};

pub struct ColorPass {
    color_uniform: ColorFxUniform,
    color_buffer: wgpu::Buffer,
    io_uniform: FxIOUniform,
    io_bg: wgpu::BindGroup,
    color_bg: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
    update_uniform: bool,
}

impl ColorPass {
    pub fn compute_tonemap<'a>(
        &'a self,
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        //c_pass.set_pipeline(&self.split_pipeline);
        //c_pass.set_bind_group(0, fx_state.bind_group(ping_pong), &[]);
        //c_pass.set_bind_group(1, &self.io_bindgroup, &[]);
        //c_pass.set_bind_group(2, &blur_bg, &[]);
        //c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);

        //ping_pong.swap(&self.io_uniform);
    }
}
