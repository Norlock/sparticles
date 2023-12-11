use crate::model::GfxState;
use egui_wgpu::wgpu::{self};
use std::{borrow::Cow, fs, path::PathBuf};

pub const SDR_PBR: &str = "pbr/pbr.wgsl";
pub const SDR_TONEMAPPING: &str = "pbr/tonemapping.wgsl";
pub const DECLARATIONS: &str = "declarations.wgsl";
pub const DIR_HAS_LIGHTS: &str = "HAS_LIGHTS";

pub struct ShaderLocation<'a> {
    pub path: PathBuf,
    pub filenames: &'a [&'a str],
}

fn finalize_shader(shader_str: String, if_directives: &[&str]) -> String {
    let mut append_line = true;
    let mut result = String::new();

    for line_raw in shader_str.lines() {
        let line = line_raw.trim();

        if let Some(stripped) = line.strip_prefix("#if") {
            let key = stripped.trim();
            append_line = if_directives.contains(&key);
        } else if line.starts_with("#else") {
            append_line = !append_line;
        } else if line.starts_with("#endif") {
            append_line = true;
        } else if append_line {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

pub struct ShaderOptions<'a> {
    pub files: &'a [&'a str],
    pub if_directives: &'a [&'a str],
    pub label: &'a str,
}

impl GfxState {
    /// Uses builtin path /src/shaders/
    pub fn create_shader_builtin(&self, options: ShaderOptions) -> wgpu::ShaderModule {
        let device = &self.device;
        let mut shader_str = String::new();
        let all_files = [&["declarations.wgsl"], options.files].concat();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/shaders/dummy.txt");

        for filename in all_files {
            let file_path = path.with_file_name(filename);
            let error_msg = format!("Path not found: {}", filename);
            let file = fs::read_to_string(file_path).expect(&error_msg);

            let res = &finalize_shader(file, options.if_directives);
            //println!("{res}");
            shader_str += res;
        }

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(options.label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
        })
    }

    /// Automatically includes declarations.wgsl
    pub fn create_shader_custom(
        &self,
        sdr_locations: Vec<ShaderLocation>,
        label: &str,
    ) -> wgpu::ShaderModule {
        let device = &self.device;
        let mut shader_str = include_str!("declarations.wgsl").to_string();

        for mut sdr_location in sdr_locations {
            let path = &mut sdr_location.path;

            for filename in sdr_location.filenames {
                path.set_file_name(filename);
                let file = fs::read_to_string(path.clone()).unwrap();

                shader_str += &file;
            }
        }

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_str)),
        })
    }
}
