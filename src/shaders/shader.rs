use egui_wgpu_backend::wgpu;
use std::borrow::Cow;

pub fn create_shader(shader_str: &str, device: &wgpu::Device, label: &str) -> wgpu::ShaderModule {
    let declarations = include_str!("declarations.wgsl");
    let shader_str = format!("{}{}", declarations, shader_str);

    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
    })
}
