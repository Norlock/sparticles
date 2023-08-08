use std::borrow::Cow;

use egui_wgpu::wgpu;

use crate::traits::CustomShader;

impl CustomShader for wgpu::Device {
    fn create_shader(&self, shader_str_raw: &str, label: &str) -> wgpu::ShaderModule {
        let declarations = include_str!("declarations.wgsl");
        let shader_str = format!("{}{}", declarations, shader_str_raw);

        self.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
        })
    }
}
