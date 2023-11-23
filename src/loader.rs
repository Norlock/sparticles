use crate::model::material::MaterialCtx;
use crate::model::{GfxState, Material, Mesh, ModelVertex};
use crate::texture::TexType;
use crate::util::ID;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use std::collections::HashMap;
use std::path::PathBuf;

pub const CIRCLE_MESH_ID: &'static str = "circle-mesh";
pub const CIRCLE_MAT_ID: &'static str = "circle-mat";
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

    Ok(gfx_state.tex_from_bytes(&data, true))
}

impl Model {
    pub fn load_builtin(gfx_state: &GfxState) -> Self {
        let mut meshes = HashMap::new();
        meshes.insert(CIRCLE_MESH_ID.to_string(), Mesh::circle(gfx_state));

        let materials = Material::create_builtin(gfx_state);

        Self {
            id: BUILTIN_ID.to_string(),
            materials,
            meshes,
        }
    }

    fn get_parents<'a>(
        gltf: &'a gltf::Gltf,
        node: &gltf::Node<'a>,
        list: &mut Vec<gltf::Node<'a>>,
    ) {
        for other in gltf.nodes() {
            for child in other.children() {
                if node.index() == child.index() {
                    list.push(child.clone());
                    Self::get_parents(gltf, &child, list);
                    break;
                }
            }
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

        let fetch_sampler = |sampler_data: gltf::texture::Sampler<'_>| -> wgpu::Sampler {
            let min_filter = match &sampler_data.min_filter() {
                Some(gltf::texture::MinFilter::Linear) => wgpu::FilterMode::Linear,
                Some(gltf::texture::MinFilter::Nearest) => wgpu::FilterMode::Nearest,
                Some(gltf::texture::MinFilter::LinearMipmapLinear) => wgpu::FilterMode::Linear,
                Some(gltf::texture::MinFilter::NearestMipmapNearest) => wgpu::FilterMode::Nearest,
                Some(gltf::texture::MinFilter::LinearMipmapNearest) => todo!(),
                Some(gltf::texture::MinFilter::NearestMipmapLinear) => todo!(),
                None => wgpu::FilterMode::default(),
            };

            let mag_filter = match &sampler_data.mag_filter() {
                Some(gltf::texture::MagFilter::Linear) => wgpu::FilterMode::Linear,
                Some(gltf::texture::MagFilter::Nearest) => wgpu::FilterMode::Nearest,
                None => wgpu::FilterMode::default(),
            };

            let get_wrapping_mode = |wrap: gltf::texture::WrappingMode| match wrap {
                gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
                gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
                gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            };

            device.create_sampler(&wgpu::SamplerDescriptor {
                label: sampler_data.name(),
                min_filter,
                mag_filter,
                address_mode_u: get_wrapping_mode(sampler_data.wrap_s()),
                address_mode_v: get_wrapping_mode(sampler_data.wrap_t()),
                ..Default::default()
            })
        };

        let fetch_texture = |img: gltf::image::Image<'_>, s_rgb: bool| -> wgpu::Texture {
            match img.source() {
                gltf::image::Source::View { view, mime_type: _ } => {
                    let start = view.offset();
                    let end = start + view.length();
                    let buf_idx = view.buffer().index();

                    gfx_state.tex_from_bytes(&buffer_data[buf_idx][start..end], s_rgb)
                }
                gltf::image::Source::Uri { uri, mime_type: _ } => {
                    pollster::block_on(load_texture(uri, gfx_state)).expect("Can't load diffuse")
                }
            }
        };

        let mut materials: HashMap<ID, Material> = HashMap::new();

        for tex in gltf.accessors() {
            //println!("tex {:?}", tex.name());
        }

        for (i, material) in gltf.materials().enumerate() {
            let albedo_tex: wgpu::Texture;
            let albedo_s: wgpu::Sampler;
            let metallic_roughness_tex: wgpu::Texture;
            let metallic_roughness_s: wgpu::Sampler;
            let normal_tex: wgpu::Texture;
            let normal_s: wgpu::Sampler;
            let emissive_tex: wgpu::Texture;
            let emissive_s: wgpu::Sampler;
            let ao_tex: wgpu::Texture;
            let ao_s: wgpu::Sampler;
            let cull_mode;
            //let y_sign;

            if material.double_sided() {
                cull_mode = None;
            } else {
                cull_mode = Some(wgpu::Face::Back);
            }

            let pbr = material.pbr_metallic_roughness();
            let normal_scale: f32;
            let emissive_factor = material.emissive_factor();
            let metallic_factor = pbr.metallic_factor();
            let roughness_factor = pbr.roughness_factor();

            if let Some(tex_data) = pbr.base_color_texture() {
                let tex = tex_data.texture();
                albedo_tex = fetch_texture(tex.source(), true);
                albedo_s = fetch_sampler(tex.sampler());
            } else {
                albedo_tex = gfx_state.create_builtin_tex(TexType::White);
                albedo_s = gfx_state.create_sampler();
            }

            if let Some(tex_data) = pbr.metallic_roughness_texture() {
                let tex = tex_data.texture();
                metallic_roughness_tex = fetch_texture(tex.source(), true);
                metallic_roughness_s = fetch_sampler(tex.sampler());
                println!("Contains metallic_roughness_tex");
            } else {
                metallic_roughness_tex = gfx_state.create_builtin_tex(TexType::White);
                metallic_roughness_s = gfx_state.create_sampler();
            }

            if let Some(tex_data) = material.normal_texture() {
                let tex = tex_data.texture();
                normal_tex = fetch_texture(tex.source(), false);
                normal_s = fetch_sampler(tex.sampler());
                normal_scale = tex_data.scale();
                println!("Contains normal_tex");
            } else {
                normal_tex = gfx_state.create_builtin_tex(TexType::Normal);
                normal_s = gfx_state.create_sampler();
                normal_scale = 1.0;
            }

            if let Some(tex_data) = material.emissive_texture() {
                let tex = tex_data.texture();
                emissive_tex = fetch_texture(tex.source(), true);
                emissive_s = fetch_sampler(tex.sampler());
                println!("contains emissive_tex");
            } else {
                emissive_tex = gfx_state.create_builtin_tex(TexType::White);
                emissive_s = gfx_state.create_sampler();
            }

            if let Some(tex_data) = material.occlusion_texture() {
                let tex = tex_data.texture();
                ao_tex = fetch_texture(tex.source(), true);
                ao_s = fetch_sampler(tex.sampler());
                println!("contains occlusion_texture");
            } else {
                ao_tex = gfx_state.create_builtin_tex(TexType::White);
                ao_s = gfx_state.create_sampler();
            }

            let id = material
                .name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| format!("material-{}", i));

            println!("Importing material: {:?}", &id);

            // todo add normal_scale
            materials.insert(
                id,
                Material::new(
                    MaterialCtx {
                        albedo_tex,
                        albedo_s,
                        emissive_tex,
                        emissive_s,
                        metallic_roughness_tex,
                        metallic_roughness_s,
                        roughness_factor,
                        metallic_factor,
                        emissive_factor,
                        normal_tex,
                        normal_s,
                        normal_scale,
                        ao_tex,
                        ao_s,
                        cull_mode,
                    },
                    gfx_state,
                ),
            );
        }

        for _sampler in gltf.samplers() {
            //println!("sampler: {:?}", sampler);
        }

        let mut meshes: HashMap<ID, Mesh> = HashMap::new();

        for scene in gltf.scenes() {
            for node in scene.nodes() {
                let mut model = glam::Mat4::from_cols_array_2d(&node.transform().matrix());

                let mut parents = vec![];
                Self::get_parents(&gltf, &node, &mut parents);

                for parent in parents {
                    println!(
                        "parent: {} from {}",
                        parent.name().unwrap_or("parent"),
                        node.name().unwrap_or("child")
                    );
                    model *= glam::Mat4::from_cols_array_2d(&parent.transform().matrix());
                }

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
                                    tangent: Default::default(),
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

                        if let Some(tangents) = reader.read_tangents() {
                            for (i, tangent) in tangents.enumerate() {
                                vertices[i].tangent = tangent;
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
                            model,
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

//let a = glam::Mat4::from_euler(glam::EulerRot::default(), 0., 0., 0.);
//let b = glam::Mat4::from_translation(glam::Vec3::new(10., 20., 30.));
//let c = glam::Mat4::from_scale(glam::Vec3::splat(3.0));

//let quat = glam::Quat::from_euler(glam::EulerRot::default(), 10., 20., 30.);
//let d = glam::Mat4::from_scale_rotation_translation(
//glam::Vec3::new(5., 5., 5.),
//quat,
//glam::Vec3::new(50., 50., 50.),
//);

//println!("rot mat {:?}", a);
//println!("tran mat {:?}", b);
//println!("scale mat {:?}", c);
//println!("rot scale tran mat {:?}", d);
