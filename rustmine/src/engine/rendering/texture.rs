use crate::engine::rendering::mipmap_pipeline::RenderPipelineBuilder;
use anyhow::*;
use image::GenericImageView;

pub(crate) struct Mipmapper {
    blit_mipmap: wgpu::RenderPipeline,
    blit_sampler: wgpu::Sampler,
}

impl Mipmapper {
    /// Creates a `Mipmapper` with a blit (Block Image Transfer) render pipeline and a bilinear sampler,
    /// used to downsample each mip level from the previous one.
    pub fn new(device: &wgpu::Device) -> Self {
        let blit_shader = wgpu::include_wgsl!("../shaders/blit.wgsl");
        let blit_format = wgpu::TextureFormat::Rgba8Unorm;
        let blit_mipmap = RenderPipelineBuilder::new()
            .vertex_shader(blit_shader.clone())
            .fragment_shader(blit_shader.clone())
            .cull_mode(Some(wgpu::Face::Back))
            .color_solid(blit_format)
            .build(device)
            .unwrap();
        let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            min_filter: wgpu::FilterMode::Nearest,
            mag_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            blit_mipmap,
            blit_sampler,
        }
    }

    /// Generates all mip levels for `texture` by repeatedly blitting each level into the next.
    ///
    /// If the texture lacks `RENDER_ATTACHMENT` usage, a temporary texture is used as an
    /// intermediate target and the results are copied back afterward.
    ///
    /// # Errors
    /// Returns an error if the texture format is not `Rgba8Unorm` or `Rgba8UnormSrgb`.
    pub fn blit_mipmaps(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
    ) -> Result<()> {
        match texture.format() {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {}
            _ => bail!("Unsupported format {:?}", texture.format()),
        }

        if texture.mip_level_count() == 1 {
            return Ok(());
        }

        let mut encoder = device.create_command_encoder(&Default::default());

        let (mut src_view, maybe_temp) = if texture
            .usage()
            .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            (
                texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(texture.format().remove_srgb_suffix()),
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    ..Default::default()
                }),
                None,
            )
        } else {
            let temp = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Mipmapper::compute_mipmaps::temp"),
                size: texture.size(),
                mip_level_count: texture.mip_level_count(),
                sample_count: texture.sample_count(),
                dimension: texture.dimension(),
                format: texture.format().remove_srgb_suffix(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            encoder.copy_texture_to_texture(
                texture.as_image_copy(),
                temp.as_image_copy(),
                temp.size(),
            );

            (
                temp.create_view(&wgpu::TextureViewDescriptor {
                    mip_level_count: Some(1),
                    ..Default::default()
                }),
                Some(temp),
            )
        };

        for mip in 1..texture.mip_level_count() {
            let dst_view = src_view
                .texture()
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(texture.format().remove_srgb_suffix()),
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                });

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.blit_mipmap.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                    },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.blit_mipmap);
            pass.set_bind_group(0, &texture_bind_group, &[]);
            pass.draw(0..3, 0..1);

            src_view = dst_view;
        }

        if let Some(temp) = maybe_temp {
            let mut size = temp.size();
            for mip_level in 0..temp.mip_level_count() {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        mip_level,
                        ..temp.as_image_copy()
                    },
                    wgpu::TexelCopyTextureInfo {
                        mip_level,
                        ..texture.as_image_copy()
                    },
                    size,
                );

                size.width /= 2;
                size.height /= 2;
            }
        }

        queue.submit([encoder.finish()]);

        Ok(())
    }
}

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a `Depth32Float` texture sized to the surface, with a `LessEqual` compare sampler,
    /// used for depth testing during rendering.
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[Self::DEPTH_FORMAT],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    /// Decodes `bytes` as an image and uploads it as a GPU texture via `from_image`.
    #[allow(dead_code)]
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    /// Converts a `DynamicImage` to an `Rgba8UnormSrgb` GPU texture with a full mip chain,
    /// using nearest-neighbor sampling. Mip levels are generated via `Mipmapper`.
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let dimensions = img.dimensions();
        let rgba = img.to_rgba8();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let mip_level_count = rgba.width().min(rgba.height()).ilog2() + 1;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let mipmapper = Mipmapper::new(device);
        mipmapper.blit_mipmaps(device, queue, &texture)?;

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
