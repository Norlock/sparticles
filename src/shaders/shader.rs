use std::{borrow::Cow, fs};

use egui_wgpu::wgpu;

use crate::traits::CustomShader;

impl CustomShader for wgpu::Device {
    fn create_shader(&self, filename: &str, label: &str) -> wgpu::ShaderModule {
        let declarations = include_str!("declarations.wgsl");
        let file = get_file(filename);

        let shader_str = format!("{}{}", declarations, file);

        self.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
        })
    }
}

fn get_file(filename: &str) -> String {
    let path = format!("src/shaders/{}", filename);
    fs::read_to_string(path).expect("Shader file doesn't exist")
}
