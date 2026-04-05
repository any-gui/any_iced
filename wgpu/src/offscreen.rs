use std::num::NonZeroU64;
use crate::core::Size;

#[derive(Clone)]
pub enum OffscreenState {
    Empty,
    Ready(OffscreenTarget),
}
impl OffscreenState {
    pub(crate) fn ensure(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) {
        match self {
            OffscreenState::Empty => {
                let mut target = OffscreenTarget::new(device, target_format, width, height);
                target.prepare(
                    device,
                    encoder,
                    belt,
                    target_format,
                    width,
                    height,
                );
                *self = Self::Ready(
                    OffscreenTarget::new(device, target_format, width, height)
                );
            }
            OffscreenState::Ready(r) => {
                r.prepare(
                    device,
                    encoder,
                    belt,
                    target_format,
                    width,
                    height,
                )
            }
        }
    }

    pub(crate) fn get_texture_view(&self) -> Option<&wgpu::TextureView> {
        match self {
            OffscreenState::Empty => None,
            OffscreenState::Ready(r) => {
                Some(&r.texture_view)
            }
        }
    }

    pub(crate) fn get_screen_target_bind_group(&self) -> Option<&wgpu::BindGroup> {
        match self {
            OffscreenState::Empty => None,
            OffscreenState::Ready(r) => {
                Some(&r.texture_bind_group)
            }
        }
    }

    pub(crate) fn render<'a>(
        &self,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        match self {
            OffscreenState::Empty => {}
            OffscreenState::Ready(r) => {
                r.render(render_pass)
            }
        }
    }
}
#[derive(Clone)]
pub struct OffscreenTarget {
    pub(crate) window_size: Size<u32>,
    pub(crate) buffer_size: Size<u32>,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) uniform_buffer: wgpu::Buffer,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) constant_layout: wgpu::BindGroupLayout,
    pub(crate) constant_bind_group: wgpu::BindGroup,
    pub(crate) texture_layout: wgpu::BindGroupLayout,
    pub(crate) texture_bind_group: wgpu::BindGroup,
    pub(crate) blit_pipeline: wgpu::RenderPipeline,
}

impl OffscreenTarget {

    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let window_size = Size::new(width, height);
        let (
            texture,
            texture_view,
            texture_bind_group_layout,
            texture_bind_group,
            buffer_size
        ) = Self::alloc_texture(
            device,
            target_format,
            Size::new(0, 0),
            window_size,
        );
        let (
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
            sampler
        ) = Self::alloc_uniform(device);
        let pipeline = Self::alloc_pipeline(
            device,
            &uniform_bind_group_layout,
            &texture_bind_group_layout,
            target_format
        );
        Self {
            window_size,
            buffer_size,
            format: target_format,
            texture,
            texture_view,
            uniform_buffer,
            sampler,
            constant_layout: uniform_bind_group_layout,
            constant_bind_group: uniform_bind_group,
            texture_layout: texture_bind_group_layout,
            texture_bind_group,
            blit_pipeline: pipeline,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) {
        let new_size = Size::new(width, height);
        if width > self.buffer_size.width || height > self.buffer_size.height {
            let (
                texture,
                texture_view,
                texture_bind_group_layout,
                texture_bind_group,
                buffer_size
            ) = Self::alloc_texture(
                device,
                target_format,
                Size::new(0, 0),
                new_size,
            );
            self.texture = texture;
            self.texture_view = texture_view;
            self.texture_layout = texture_bind_group_layout;
            self.texture_bind_group = texture_bind_group;
            self.buffer_size = buffer_size;
        }
        if width > self.window_size.width || height > self.window_size.height {
            self.window_size = new_size;
            self.prepare_uniform(
                device,
                encoder,
                belt
            );
        }
    }

    pub fn render<'a>(
        &self,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.blit_pipeline);
        render_pass.set_bind_group(0, &self.constant_bind_group, &[]);
        render_pass.set_bind_group(
            1,
            &self.texture_bind_group,
            &[],
        );
        render_pass.draw(0..6, 0..1);
    }

    pub fn prepare_uniform(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
    ) {
        let ratio = UVRatio {
            u: self.window_size.width as f32 / self.buffer_size.width as f32,
            v: self.window_size.height as f32 / self.buffer_size.height as f32,
            _padding: [0.0; 2],
        };
        belt.write_buffer(
            encoder,
            &self.uniform_buffer,
            0,
            NonZeroU64::new(std::mem::size_of::<UVRatio>() as u64)
                .expect("non-empty ratio"),
            device,
        )
            .copy_from_slice(bytemuck::bytes_of(&ratio));
    }

    /// Create:
    /// Texture
    /// TextureView
    /// Texture Bind Group Layout
    /// Texture Bind Group
    pub fn alloc_texture(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        current_size: Size<u32>,
        new_size: Size<u32>,
    ) -> (wgpu::Texture, wgpu::TextureView,wgpu::BindGroupLayout, wgpu::BindGroup,Size<u32>) {
        let bw = next_buffer_size(current_size.width, new_size.width);
        let bh = next_buffer_size(current_size.height, new_size.height);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen_texture"),
            size: wgpu::Extent3d {
                width: bw,
                height: bh,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: target_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&Default::default());

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::blit:texture layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: false,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::blit::texture bind group"),
            layout: &texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
        });
        (texture,texture_view,texture_layout,texture_bind_group,Size::new(bw, bh))
    }

    /// Pipeline
    pub fn alloc_pipeline(
        device: &wgpu::Device,
        constant_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu::blit:pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[constant_layout, texture_layout],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu::blit:pipeline shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader/blit.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu::blit:pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options:
                    wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(
                            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
                        ),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options:
                    wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        pipeline
    }

    /// Create:
    /// Uniform Buffer
    /// Uniform bind group layout
    /// uniform bind group
    /// sampler
    fn alloc_uniform(
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup, wgpu::Sampler) {
        let constant_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::blit:uniforms layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::NonFiltering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let ratio = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu::blit::uniform buffer"),
            size: std::mem::size_of::<UVRatio>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let sampler =
            device.create_sampler(&wgpu::SamplerDescriptor::default());

        let constants_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::blit::uniforms bind group"),
            layout: &constant_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ratio.as_entire_binding(),
                },
            ],
        });
        (ratio,constant_layout,constants_bind_group,sampler)
    }
}

fn next_buffer_size(old: u32, required: u32) -> u32 {
    if old == 0 {
        return required.next_power_of_two();
    }
    if required <= old {
        old
    } else {
        let grown = old * 3 / 2;
        grown.max(required)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct UVRatio {
    u: f32,
    v: f32,
    // Padding field for 16-byte alignment.
    // See https://docs.rs/wgpu/latest/wgpu/struct.DownlevelFlags.html#associatedconstant.BUFFER_BINDINGS_NOT_16_BYTE_ALIGNED
    _padding: [f32; 2],
}