use std::io::{BufReader, Cursor};

use crate::engine::rendering::{model, texture};
use wgpu::util::DeviceExt;

/// Returns the text asset `file_name` embedded at compile time via `include_str!`.
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let txt = match file_name {
        // Obj files
        "grass.obj" => include_str!("../../assets/textures/grass.obj"),
        "dirt.obj" => include_str!("../../assets/textures/dirt.obj"),
        "cobblestone.obj" => include_str!("../../assets/textures/cobblestone.obj"),
        "stone.obj" => include_str!("../../assets/textures/stone.obj"),

        "oak.obj" => include_str!("../../assets/textures/oak.obj"),
        "birch.obj" => include_str!("../../assets/textures/birch.obj"),
        "spruce.obj" => include_str!("../../assets/textures/spruce.obj"),

        "leaves_oak.obj" => include_str!("../../assets/textures/leaves_oak.obj"),
        "leaves_birch.obj" => include_str!("../../assets/textures/leaves_birch.obj"),
        "leaves_spruce.obj" => include_str!("../../assets/textures/leaves_spruce.obj"),

        "tall_grass.obj" => include_str!("../../assets/textures/tall_grass.obj"),

        // Mtl files
        "grass.mtl" => include_str!("../../assets/textures/grass.mtl"),
        "dirt.mtl" => include_str!("../../assets/textures/dirt.mtl"),
        "cobblestone.mtl" => include_str!("../../assets/textures/cobblestone.mtl"),
        "stone.mtl" => include_str!("../../assets/textures/stone.mtl"),

        "oak.mtl" => include_str!("../../assets/textures/oak.mtl"),
        "birch.mtl" => include_str!("../../assets/textures/birch.mtl"),
        "spruce.mtl" => include_str!("../../assets/textures/spruce.mtl"),

        "leaves_oak.mtl" => include_str!("../../assets/textures/leaves_oak.mtl"),
        "leaves_birch.mtl" => include_str!("../../assets/textures/leaves_birch.mtl"),
        "leaves_spruce.mtl" => include_str!("../../assets/textures/leaves_spruce.mtl"),

        "tall_grass.mtl" => include_str!("../../assets/textures/tall_grass.mtl"),
        other => anyhow::bail!("unknown text asset: {other}"),
    };
    Ok(txt.to_string())
}

/// Returns the binary asset `file_name` embedded at compile time via `include_bytes!`.
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let data: &[u8] = match file_name {
        "atlas-grass.png" => include_bytes!("../../assets/textures/atlas-grass.png"),
        "atlas-dirt.png" => include_bytes!("../../assets/textures/atlas-dirt.png"),
        "atlas-cobblestone.png" => include_bytes!("../../assets/textures/atlas-cobblestone.png"),
        "atlas-stone.png" => include_bytes!("../../assets/textures/atlas-stone.png"),
        "atlas-oak.png" => include_bytes!("../../assets/textures/atlas-oak.png"),
        "atlas-birch.png" => include_bytes!("../../assets/textures/atlas-birch.png"),
        "atlas-spruce.png" => include_bytes!("../../assets/textures/atlas-spruce.png"),
        "atlas-leaves-oak.png" => include_bytes!("../../assets/textures/atlas-leaves-oak.png"),
        "atlas-leaves-birch.png" => include_bytes!("../../assets/textures/atlas-leaves-birch.png"),
        "atlas-leaves-spruce.png" => {
            include_bytes!("../../assets/textures/atlas-leaves-spruce.png")
        }
        "atlas-tall-grass.png" => include_bytes!("../../assets/textures/atlas-tall-grass.png"),
        other => anyhow::bail!("unknown binary asset: {other}"),
    };
    Ok(data.to_vec())
}

/// Loads an image from the assets directory and uploads it as a GPU texture.
pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

/// Loads an `.obj` model and its `.mtl` materials from the assets directory.
///
/// Vertices are triangulated and use a single index buffer. Tex coords are
/// V-flipped to convert from OBJ convention (bottom-left origin) to WGPU's
/// (top-left origin). Normals default to zero if absent in the source mesh.
pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture, device, queue).await?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3] / 2.0,
                                m.mesh.positions[i * 3 + 1] / 2.0,
                                m.mesh.positions[i * 3 + 2] / 2.0,
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3] / 2.0,
                                m.mesh.positions[i * 3 + 1] / 2.0,
                                m.mesh.positions[i * 3 + 2] / 2.0,
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            log::debug!("Mesh: {}", m.name);

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}
