use egui_wgpu::wgpu::{self, util::DeviceExt};

use crate::model::{GfxState, Material, Mesh, ModelVertex};

pub struct Loader {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub async fn load_binary(filename: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/assets/models")
        .join(filename);

    println!("path: {:?}", &path);
    let data = std::fs::read(path)?;

    Ok(data)
}

pub async fn load_texture(filename: &str, gfx_state: &GfxState) -> anyhow::Result<wgpu::Texture> {
    println!("file: {:?}", filename);
    let data = load_binary(filename).await?;

    let res = Ok(gfx_state.diffuse_from_bytes(&data));
    println!("success");

    res
    //texture::Texture::from_bytes(device, queue, &data, file_name)
}

impl Loader {
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
                    let bin: Vec<u8> = std::fs::read(uri)?;
                    buffer_data.push(bin);
                    println!("Found uri not saving")
                }
            }
        }

        let mut materials = Vec::new();

        for material in gltf.materials() {
            println!("Looping thru materials");
            let pbr = material.pbr_metallic_roughness();
            let base_color_texture = pbr.base_color_texture();

            let texture_source = base_color_texture
                .map(|tex| {
                    println!("Grabbing diffuse tex");
                    //dbg!(&tex.texture().source());
                    tex.texture().source().source()
                })
                .expect("Expect diffuse texture");

            match texture_source {
                gltf::image::Source::View { view, mime_type: _ } => {
                    println!("{:?}", view.offset());
                    let offset = view.offset();
                    let diffuse_texture = gfx_state.diffuse_from_bytes(
                        &buffer_data[view.buffer().index()][offset..offset + view.length()],
                    );

                    materials.push(Material {
                        name: material.name().unwrap_or("Default Material").to_string(),
                        texture: diffuse_texture,
                    });
                }
                gltf::image::Source::Uri { uri, mime_type: _ } => {
                    let diffuse_texture = pollster::block_on(load_texture(uri, gfx_state))?;

                    materials.push(Material {
                        name: material.name().unwrap_or("Default Material").to_string(),
                        texture: diffuse_texture,
                    });
                }
            };
        }

        let mut meshes = Vec::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                if let Some(mesh) = node.mesh() {
                    println!("is a mesh");
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

                    meshes.push(Mesh {
                        label: filename.to_string(),
                        vertices,
                        indices,
                        vertex_buffer,
                        index_buffer,
                    });
                } else {
                    println!("Not a mesh! {}", node.name().unwrap_or("no name"));
                }
            }
        }

        println!("mat {:?}", materials.len());
        println!("mesh {:?}", meshes.len());

        Ok(Self { meshes, materials })
    }
}
