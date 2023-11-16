use crate::model::material::MaterialCtx;
use crate::model::{GfxState, Material, Mesh, ModelVertex};
use crate::util::ID;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use std::collections::HashMap;
use std::path::PathBuf;

pub const CIRCLE_ID: &'static str = "circle";
pub const DEFAULT_MATERIAL_ID: &'static str = "white-img";
pub const BUILTIN_ID: &'static str = "builtin";

pub struct Model {
    pub id: ID,
    pub materials: HashMap<ID, Material>,
    pub meshes: HashMap<ID, Mesh>,
}

async fn load_binary(filename: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/assets/models")
        .join(filename);

    println!("path: {:?}", &path);
    let data = std::fs::read(path)?;

    Ok(data)
}

async fn load_texture(filename: &str, gfx_state: &GfxState) -> anyhow::Result<wgpu::Texture> {
    println!("file: {:?}", filename);
    let data = load_binary(filename).await?;

    Ok(gfx_state.diffuse_from_bytes(&data))
}

impl Model {
    pub fn load_builtin(gfx_state: &GfxState) -> Self {
        // TODO create default material
        let mut texture_image = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        texture_image.push("src/assets/textures/1x1.png");

        let mut meshes = HashMap::new();
        meshes.insert(CIRCLE_ID.to_string(), Mesh::circle(gfx_state));

        let materials = Material::create_builtin(gfx_state);

        Self {
            id: BUILTIN_ID.to_string(),
            materials,
            meshes,
        }
    }

    pub fn load_gltf(gfx_state: &GfxState, filename: &str) -> anyhow::Result<Self> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/assets/models")
            .join(filename);

        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let gltf = gltf::Gltf::from_reader(reader)?;
        let device = &gfx_state.device;

        // Load buffers
        let mut buffer_data: Vec<Vec<u8>> = Vec::new();

        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Bin => {
                    if let Some(blob) = gltf.blob.as_deref() {
                        buffer_data.push(blob.into());
                        println!("Found a bin, saving");
                    };
                }
                gltf::buffer::Source::Uri(uri) => {
                    buffer_data.push(std::fs::read(uri)?);
                    println!("Found uri not saving")
                }
            }
        }

        let fetch_texture = |tex: gltf::Texture<'_>| -> wgpu::Texture {
            match tex.source().source() {
                gltf::image::Source::View { view, mime_type: _ } => {
                    let start = view.offset();
                    let end = start + view.length();
                    let buf_idx = view.buffer().index();

                    gfx_state.diffuse_from_bytes(&buffer_data[buf_idx][start..end])
                }
                gltf::image::Source::Uri { uri, mime_type: _ } => {
                    pollster::block_on(load_texture(uri, gfx_state)).expect("Can't load diffuse")
                }
            }
        };

        let mut materials: HashMap<ID, Material> = HashMap::new();

        for (i, material) in gltf.materials().enumerate() {
            let diffuse_tex: wgpu::Texture;
            let metallic_roughness_tex: wgpu::Texture;
            let normal_tex: wgpu::Texture;
            let emissive_tex: wgpu::Texture;

            let pbr = material.pbr_metallic_roughness();

            if let Some(tex) = pbr.base_color_texture() {
                diffuse_tex = fetch_texture(tex.texture());
            } else {
                todo!("create default texture");
            }

            if let Some(tex) = pbr.metallic_roughness_texture() {
                metallic_roughness_tex = fetch_texture(tex.texture());
            } else {
                todo!("create default texture");
            }

            if let Some(tex) = material.normal_texture() {
                normal_tex = fetch_texture(tex.texture());
            } else {
                todo!("create default texture");
            }

            if let Some(tex) = material.emissive_texture() {
                emissive_tex = fetch_texture(tex.texture());
            } else {
                todo!("create default texture");
            }

            let id = material
                .name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| format!("material-{}", i));

            println!("Importing material: {:?}", &id);

            materials.insert(
                id,
                Material::new(MaterialCtx {
                    gfx_state,
                    diffuse_tex,
                    emissive_tex,
                    metallic_roughness_tex,
                    normal_tex,
                }),
            );
        }

        let mut meshes: HashMap<ID, Mesh> = HashMap::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                if let Some(mesh) = node.mesh() {
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();

                    for primitive in mesh.primitives() {
                        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                        if let Some(vertex_attribute) = reader.read_positions() {
                            vertex_attribute.for_each(|vertex| {
                                vertices.push(ModelVertex {
                                    position: vertex,
                                    uv: Default::default(),
                                    normal: Default::default(),
                                })
                            });
                        }

                        if let Some(normal_attribute) = reader.read_normals() {
                            for (i, normal) in normal_attribute.enumerate() {
                                vertices[i].normal = normal;
                            }
                        }

                        if let Some(tex_coords) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                            for (i, uv) in tex_coords.enumerate() {
                                vertices[i].uv = uv;
                            }
                        }

                        if let Some(indices_raw) = reader.read_indices() {
                            indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                        }
                    }

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Vertex Buffer", filename)),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    let index_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Index Buffer", filename)),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                    let id = mesh
                        .name()
                        .map(|name| name.to_string())
                        .unwrap_or_else(|| format!("mesh-{}", materials.len()));

                    println!("Importing mesh: {:?}", &id);

                    meshes.insert(
                        id,
                        Mesh {
                            indices,
                            vertices,
                            vertex_buffer,
                            index_buffer,
                        },
                    );
                } else {
                    println!("Not a mesh! {}", node.name().unwrap_or("no name"));
                }
            }
        }

        Ok(Self {
            id: filename.to_string(),
            materials,
            meshes,
        })
    }
}
