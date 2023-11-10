use egui_wgpu::wgpu::{self, util::DeviceExt, Texture};

use crate::model::{self, GfxState, Mesh, ModelVertex};

pub struct Loader;

impl Loader {
    pub fn load_fbx(gfx_state: &GfxState, filename: &str) -> anyhow::Result<Mesh> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/assets/models")
            .join(filename);

        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let gltf = gltf::Gltf::from_reader(reader)?;

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
                gltf::buffer::Source::Uri(_uri) => {
                    //let bin: Vec<u8> = file. std::fs::read(file)?;
                    //buffer_data.push(bin);
                    println!("Found  not saving")
                }
            }
        }

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                let mesh = node.mesh().expect("Got mesh");

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
            }
        }

        let device = &gfx_state.device;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", filename)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", filename)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut materials = Vec::new();

        for material in gltf.materials() {
            println!("Looping thru materials");
            let pbr = material.pbr_metallic_roughness();
            let base_color_texture = &pbr.base_color_texture();
            let texture_source = &pbr
                .base_color_texture()
                .map(|tex| {
                    println!("Grabbing diffuse tex");
                    dbg!(&tex.texture().source());
                    tex.texture().source().source()
                })
                .expect("texture");

            match texture_source {
                gltf::image::Source::View { view, mime_type } => {
                    let diffuse_texture = Texture::from_bytes(
                        device,
                        queue,
                        &buffer_data[view.buffer().index()],
                        file_name,
                    )
                    .expect("Couldn't load diffuse");

                    materials.push(model::Material {
                        name: material.name().unwrap_or("Default Material").to_string(),
                        diffuse_texture,
                    });
                }
                gltf::image::Source::Uri { uri, mime_type } => {
                    let diffuse_texture = load_texture(uri, device, queue).await?;

                    materials.push(model::Material {
                        name: material.name().unwrap_or("Default Material").to_string(),
                        diffuse_texture,
                    });
                }
            };
        }

        Ok(Mesh {
            label: filename.to_string(),
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        })

        //let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //label: Some(&format!("{:?} Vertex Buffer", file_name)),
        //contents: bytemuck::cast_slice(&vertices),
        //usage: wgpu::BufferUsages::VERTEX,
        //});
        //let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //label: Some(&format!("{:?} Index Buffer", file_name)),
        //contents: bytemuck::cast_slice(&indices),
        //usage: wgpu::BufferUsages::INDEX,
        //});
    }
}
