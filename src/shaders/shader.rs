use egui_wgpu::wgpu::{self};
use std::{borrow::Cow, fs, path::PathBuf};

use crate::traits::CustomShader;

pub const PBR_SDR: &str = "pbr/pbr.wgsl";
pub const TONEMAPPING_SDR: &str = "pbr/tonemapping.wgsl";
pub const DECLARATIONS: &str = "declarations.wgsl";

pub struct ShaderLocation {
    pub path: PathBuf,
    pub filenames: Vec<String>,
}

impl CustomShader for wgpu::Device {
    /// Use builtin path /src/shaders/
    fn create_shader_builtin(&self, filenames: &[&str], label: &str) -> wgpu::ShaderModule {
        let mut shader_str = String::new();
        let all_files = [&["declarations.wgsl"], filenames].concat();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(format!("src/shaders/none.txt"));

        for filename in all_files {
            let file_path = path.with_file_name(filename);
            let error_msg = format!("Path not found: {}", file_path.to_str().unwrap());
            let file = fs::read_to_string(file_path).expect(&error_msg);

            shader_str += &file;
        }

        self.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
        })
    }

    /// Automatically includes declarations.wgsl
    fn create_shader_custom(
        &self,
        sdr_locations: Vec<ShaderLocation>,
        label: &str,
    ) -> wgpu::ShaderModule {
        let mut shader_str = include_str!("declarations.wgsl").to_string();

        for mut sdr_location in sdr_locations {
            let path = &mut sdr_location.path;

            for filename in sdr_location.filenames {
                path.set_file_name(&filename);
                let file = fs::read_to_string(path.clone()).unwrap();

                shader_str += &file;
            }
        }

        self.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
        })
    }
}
