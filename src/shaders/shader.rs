use std::{borrow::Cow, fs, path::PathBuf};

use egui_wgpu::wgpu;

use crate::traits::CustomShader;

impl CustomShader for wgpu::Device {
    fn create_shader(&self, filename: &str, label: &str) -> wgpu::ShaderModule {
        let declarations = include_str!("declarations.wgsl");
        let noise = include_str!("noise.wgsl");
        let file = get_file(filename);

        let shader_str = format!("{}{}{}", declarations, noise, file);
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push(format!("debug/{}", filename));

        let _ = fs::write(
            dir.to_str().expect("Can't create debug shader"),
            &shader_str,
        );

        self.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
        })
    }
}

fn get_file(filename: &str) -> String {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push(format!("src/shaders/{}", filename));

    let path = dir.to_str().expect("shader file doesn't exist");
    fs::read_to_string(path).expect("Shader file doesn't exist")
}
