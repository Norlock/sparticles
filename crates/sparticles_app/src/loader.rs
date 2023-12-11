use crate::model::material::MaterialCtx;
use crate::model::{GfxState, Material, Mesh, ModelVertex};
use crate::texture::{TexType, TextureHandler};
use crate::util::ID;
use async_std::sync::RwLock;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use std::collections::HashMap;
use std::sync::Arc;

pub const CIRCLE_MESH_ID: &str = "circle-mesh";
pub const CIRCLE_MAT_ID: &str = "circle-mat";
pub const BUILTIN_ID: &str = "builtin";

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

async fn load_texture(
    filename: &str,
    gfx: &Arc<RwLock<GfxState>>,
) -> anyhow::Result<wgpu::Texture> {
    println!("file: {:?}", filename);
    let data = load_binary(filename).await?;

    Ok(TextureHandler::tex_from_bytes(gfx, &data, true).await)
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

    pub async fn load_gltf(gfx: &Arc<RwLock<GfxState>>, filename: &str) -> anyhow::Result<Self> {
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
                gltf::buffer::Source::Uri(uri) => {
                    buffer_data.push(std::fs::read(uri)?);
                    println!("Found uri not saving")
                }
            }
        }

        async fn fetch_sampler(
            sampler_data: gltf::texture::Sampler<'_>,
            gfx: &Arc<RwLock<GfxState>>,
        ) -> wgpu::Sampler {
            let default_sampler = wgpu::SamplerDescriptor::default();

            let (min_filter, mipmap_filter) = match &sampler_data.min_filter() {
                Some(gltf::texture::MinFilter::Linear) => {
                    (wgpu::FilterMode::Linear, default_sampler.mipmap_filter)
                }
                Some(gltf::texture::MinFilter::Nearest) => {
                    (wgpu::FilterMode::Nearest, default_sampler.mipmap_filter)
                }
                Some(gltf::texture::MinFilter::LinearMipmapLinear) => {
                    (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear)
                }
                Some(gltf::texture::MinFilter::NearestMipmapNearest) => {
                    (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
                }
                Some(gltf::texture::MinFilter::LinearMipmapNearest) => {
                    (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest)
                }
                Some(gltf::texture::MinFilter::NearestMipmapLinear) => {
                    (wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear)
                }
                None => (default_sampler.min_filter, default_sampler.mipmap_filter),
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

            let device = &gfx.write().await.device;

            device.create_sampler(&wgpu::SamplerDescriptor {
                label: sampler_data.name(),
                min_filter,
                mag_filter,
                mipmap_filter,
                address_mode_u: get_wrapping_mode(sampler_data.wrap_s()),
                address_mode_v: get_wrapping_mode(sampler_data.wrap_t()),
                ..Default::default()
            })
        }

        async fn fetch_texture(
            img: gltf::image::Image<'_>,
            s_rgb: bool,
            buffer_data: &mut [Vec<u8>],
            gfx: &Arc<RwLock<GfxState>>,
        ) -> wgpu::Texture {
            match img.source() {
                gltf::image::Source::View { view, mime_type: _ } => {
                    let start = view.offset();
                    let end = start + view.length();
                    let buf_idx = view.buffer().index();

                    TextureHandler::tex_from_bytes(gfx, &buffer_data[buf_idx][start..end], s_rgb)
                        .await
                }
                gltf::image::Source::Uri { uri, mime_type: _ } => {
                    load_texture(uri, gfx).await.expect("Can't load diffuse")
                }
            }
        }

        let mut materials: HashMap<ID, Material> = HashMap::new();

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
            let cull_mode = Some(wgpu::Face::Back);

            let pbr = material.pbr_metallic_roughness();

            if let Some(tex_data) = pbr.base_color_texture() {
                let tex = tex_data.texture();
                albedo_tex = fetch_texture(tex.source(), true, &mut buffer_data, gfx).await;
                albedo_s = fetch_sampler(tex.sampler(), gfx).await;
                println!("Contains albedo tex");
            } else {
                let gfx = &gfx.read().await;
                albedo_tex = gfx.create_builtin_tex(TexType::White);
                albedo_s = gfx.create_sampler();
            }

            if let Some(tex_data) = pbr.metallic_roughness_texture() {
                let tex = tex_data.texture();
                metallic_roughness_tex =
                    fetch_texture(tex.source(), true, &mut buffer_data, gfx).await;
                metallic_roughness_s = fetch_sampler(tex.sampler(), gfx).await;
                println!("Contains metallic_roughness_tex");
            } else {
                let metallic_factor = pbr.metallic_factor();
                let roughness_factor = pbr.roughness_factor();

                let gfx = &gfx.read().await;
                metallic_roughness_tex = gfx.create_builtin_tex(TexType::Custom {
                    srgb: true,
                    value: glam::Vec4::new(metallic_factor, roughness_factor, 0., 0.),
                });
                metallic_roughness_s = gfx.create_sampler();
            }

            if let Some(tex_data) = material.normal_texture() {
                let tex = tex_data.texture();
                normal_tex = fetch_texture(tex.source(), false, &mut buffer_data, gfx).await;
                normal_s = fetch_sampler(tex.sampler(), gfx).await;
                println!("Contains normal_tex");
            } else {
                let gfx = &gfx.read().await;
                normal_tex = gfx.create_builtin_tex(TexType::Normal);
                normal_s = gfx.create_sampler();
            }

            if let Some(tex_data) = material.emissive_texture() {
                let tex = tex_data.texture();
                emissive_tex = fetch_texture(tex.source(), true, &mut buffer_data, gfx).await;
                emissive_s = fetch_sampler(tex.sampler(), gfx).await;

                if let Some(strenght) = material.emissive_strength() {
                    println!("Strength: {}", strenght);
                    // TODO with khr
                }
                println!("contains emissive_tex");
            } else {
                let gfx = &gfx.read().await;
                let vec3: glam::Vec3 = material.emissive_factor().into();
                emissive_tex = gfx.create_builtin_tex(TexType::Custom {
                    srgb: true,
                    value: vec3.extend(0.),
                });
                emissive_s = gfx.create_sampler();
            }

            if let Some(tex_data) = material.occlusion_texture() {
                let tex = tex_data.texture();
                ao_tex = fetch_texture(tex.source(), true, &mut buffer_data, gfx).await;
                ao_s = fetch_sampler(tex.sampler(), gfx).await;
                println!("contains occlusion_texture");
            } else {
                let gfx = &gfx.read().await;
                ao_tex = gfx.create_builtin_tex(TexType::White);
                ao_s = gfx.create_sampler();
            }

            let id = material
                .name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| format!("material-{}", i));

            println!("Importing material: {:?}", &id);

            let gfx = &gfx.read().await;
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
                        normal_tex,
                        normal_s,
                        ao_tex,
                        ao_s,
                        cull_mode,
                    },
                    gfx,
                ),
            );
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
                                    bitangent: Default::default(),
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
                                let tn: glam::Vec3 = glam::Vec3::from_slice(&tangent[..3]);
                                let nm: glam::Vec3 = vertices[i].normal.into();

                                vertices[i].tangent.copy_from_slice(&tangent[..3]);
                                vertices[i].bitangent = (nm.cross(tn) * tangent[3]).into();
                            }
                        }

                        if let Some(indices_raw) = reader.read_indices() {
                            indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                        }
                    }

                    let device = &gfx.read().await.device;

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
                            fs_entry_point: "fs_model".to_string(),
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
