use crate::Buffer;
use crate::core::Size;
use std::collections::HashSet;
use wgpu::util::RenderEncoder;

#[derive(Debug)]
pub struct OffscreenState {
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) uniform_buffer: Buffer<UVRatio>,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) constant_layout: wgpu::BindGroupLayout,
    pub(crate) constant_bind_group: wgpu::BindGroup,
    pub(crate) texture_layout: wgpu::BindGroupLayout,
    pub(crate) blit_pipeline: wgpu::RenderPipeline,
    // Screen Texture
    pub(crate) screen_target: OffscreenTexture,
    // Layer Texture
    pub(crate) layer_target: OffscreenTexture,
    // Window Size
    pub(crate) window_size: Size<u32>,
    // Buffer Size
    pub(crate) buffer_size: Size<u32>,
    // Layer index map
    pub(crate) layer_index_map: HashSet<usize>,
    // Use Frame Offscreen
    pub(crate) use_frame_offscreen: bool,
}

#[derive(Debug)]
pub enum OffscreenTexture {
    Empty,
    Ready(OffscreenTarget),
}

impl OffscreenTexture {
    pub fn ensure(
        &mut self,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        buffer_width: u32,
        buffer_height: u32,
    ) {
        match self {
            OffscreenTexture::Empty => {
                let mut target = OffscreenTarget::new(
                    device,
                    target_format,
                    texture_bind_group_layout,
                    buffer_width,
                    buffer_height,
                );
                target.prepare(
                    device,
                    target_format,
                    texture_bind_group_layout,
                    buffer_width,
                    buffer_height,
                );
                *self = Self::Ready(OffscreenTarget::new(
                    device,
                    target_format,
                    texture_bind_group_layout,
                    buffer_width,
                    buffer_height,
                ));
            }
            OffscreenTexture::Ready(r) => r.prepare(
                device,
                target_format,
                texture_bind_group_layout,
                buffer_width,
                buffer_height,
            ),
        }
    }
}

impl OffscreenState {
    pub(crate) fn clear(&mut self) {
        self.layer_index_map.clear();
        self.use_frame_offscreen = false;
    }

    pub(crate) fn is_layer_use_offscreen(
        &self,
        layer_index: usize,
    ) -> Option<(&wgpu::TextureView, &wgpu::BindGroup)> {
        let layer_use_offscreen = self.layer_index_map.contains(&layer_index);
        if layer_use_offscreen {
            self.get_layer_texture_view_bind_group()
        } else {
            None
        }
    }

    pub fn set_layer_index(&mut self, layer_index: usize) {
        let _ = self.layer_index_map.insert(layer_index);
    }

    pub(crate) fn ensure(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        use_screen: bool,
    ) {
        let buffer_size = Size::new(
            next_buffer_size(self.buffer_size.width, width),
            next_buffer_size(self.buffer_size.height, height),
        );
        let window_size = Size::new(width, height);
        if use_screen {
            self.screen_target.ensure(
                device,
                target_format,
                &self.texture_layout,
                buffer_size.width,
                buffer_size.height,
            );
        }
        if !self.layer_index_map.is_empty() {
            self.layer_target.ensure(
                device,
                target_format,
                &self.texture_layout,
                buffer_size.width,
                buffer_size.height,
            );
        }
        if self.window_size != window_size || self.buffer_size != buffer_size {
            self.prepare_uniform(
                device,
                encoder,
                belt,
                &window_size,
                &buffer_size,
            )
        }
        self.window_size = window_size;
        self.buffer_size = buffer_size;
        self.use_frame_offscreen = use_screen;
    }

    pub(crate) fn get_screen_texture_view_bind_group(
        &self,
    ) -> Option<(&wgpu::TextureView, &wgpu::BindGroup)> {
        match &self.screen_target {
            OffscreenTexture::Empty => None,
            OffscreenTexture::Ready(r) => {
                Some((&r.texture_view, &r.texture_bind_group))
            }
        }
    }

    pub(crate) fn get_layer_texture_view_bind_group(
        &self,
    ) -> Option<(&wgpu::TextureView, &wgpu::BindGroup)> {
        match &self.layer_target {
            OffscreenTexture::Empty => None,
            OffscreenTexture::Ready(r) => {
                Some((&r.texture_view, &r.texture_bind_group))
            }
        }
    }

    pub fn use_frame_offscreen(&self) -> bool {
        self.use_frame_offscreen
    }

    pub(crate) fn get_frame_bind_group(&self) -> Option<wgpu::BindGroup> {
        if self.use_frame_offscreen {
            match &self.screen_target {
                OffscreenTexture::Empty => None,
                OffscreenTexture::Ready(r) => {
                    Some(r.texture_bind_group.clone())
                }
            }
        } else {
            None
        }
    }

    pub(crate) fn get_frame_texture_view(&self) -> Option<&wgpu::TextureView> {
        if self.use_frame_offscreen {
            match &self.screen_target {
                OffscreenTexture::Empty => None,
                OffscreenTexture::Ready(r) => Some(&r.texture_view),
            }
        } else {
            None
        }
    }

    pub(crate) fn get_layer_bind_group(&self) -> Option<wgpu::BindGroup> {
        match &self.layer_target {
            OffscreenTexture::Empty => None,
            OffscreenTexture::Ready(r) => Some(r.texture_bind_group.clone()),
        }
    }

    pub(crate) fn get_buffer_size(&self) -> Size<u32> {
        self.buffer_size
    }

    pub(crate) fn render_to_screen<'a>(
        &self,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if let Some(bd) = self.get_frame_bind_group() {
            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_bind_group(0, &self.constant_bind_group, &[]);
            render_pass.set_bind_group(1, &bd, &[]);
            render_pass.draw(0..6, 0..1);
        }
    }

    pub(crate) fn render_to_layer<'a>(
        &self,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if let Some(bd) = self.get_layer_bind_group() {
            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_bind_group(0, &self.constant_bind_group, &[]);
            render_pass.set_bind_group(1, &bd, &[]);
            render_pass.draw(0..6, 0..1);
        }
    }

    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu::blit:texture layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });
        let (
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
            sampler,
        ) = Self::alloc_uniform(device);
        let pipeline = Self::alloc_pipeline(
            device,
            &uniform_bind_group_layout,
            &texture_layout,
            target_format,
        );
        Self {
            format: target_format,
            uniform_buffer,
            sampler,
            constant_layout: uniform_bind_group_layout,
            constant_bind_group: uniform_bind_group,
            texture_layout,
            blit_pipeline: pipeline,
            screen_target: OffscreenTexture::Empty,
            layer_target: OffscreenTexture::Empty,
            window_size: Default::default(),
            buffer_size: Default::default(),
            layer_index_map: Default::default(),
            use_frame_offscreen: false,
        }
    }

    pub fn prepare_uniform(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        window_size: &Size<u32>,
        buffer_size: &Size<u32>,
    ) {
        let ratio = UVRatio::new(
            window_size.width as f32 / buffer_size.width as f32,
            window_size.height as f32 / buffer_size.height as f32,
        );
        let _ = self
            .uniform_buffer
            .write(device, encoder, belt, 0, &[ratio]);
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
    ) -> (
        Buffer<UVRatio>,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
        wgpu::Sampler,
    ) {
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

        let ratio = Buffer::new(
            device,
            "iced_wgpu.offscreen.uniforms.uvratio",
            1,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let sampler =
            device.create_sampler(&wgpu::SamplerDescriptor::default());

        let constants_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::blit::uniforms bind group"),
                layout: &constant_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &ratio.raw,
                                offset: 0,
                                size: wgpu::BufferSize::new(
                                    std::mem::size_of::<UVRatio>() as u64,
                                ),
                            },
                        ),
                    },
                ],
            });
        (ratio, constant_layout, constants_bind_group, sampler)
    }
}
#[derive(Debug)]
pub struct OffscreenTarget {
    pub(crate) texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
    pub(crate) texture_bind_group: wgpu::BindGroup,
    pub(crate) buffer_size: Size<u32>,
}

impl OffscreenTarget {
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        buffer_width: u32,
        buffer_height: u32,
    ) -> Self {
        let buffer_size = Size::new(buffer_width, buffer_height);
        let (texture, texture_view, texture_bind_group) = Self::alloc_texture(
            device,
            target_format,
            buffer_size,
            texture_bind_group_layout,
        );

        Self {
            buffer_size,
            texture,
            texture_view,
            texture_bind_group,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        buffer_width: u32,
        buffer_height: u32,
    ) {
        let new_size = Size::new(buffer_width, buffer_height);
        if buffer_width > self.buffer_size.width
            || buffer_height > self.buffer_size.height
        {
            let (texture, texture_view, texture_bind_group) =
                Self::alloc_texture(
                    device,
                    target_format,
                    new_size,
                    texture_bind_group_layout,
                );
            self.texture = texture;
            self.texture_view = texture_view;
            self.texture_bind_group = texture_bind_group;
            self.buffer_size = new_size;
        }
    }

    /// Create:
    /// Texture
    /// TextureView
    /// Texture Bind Group Layout
    /// Texture Bind Group
    pub fn alloc_texture(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        new_size: Size<u32>,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen_texture"),
            size: wgpu::Extent3d {
                width: new_size.width,
                height: new_size.height,
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

        let texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::blit::texture bind group"),
                layout: texture_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                }],
            });
        (texture, texture_view, texture_bind_group)
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
    _padding: [f32; 62],
}

impl UVRatio {
    fn new(u: f32, v: f32) -> Self {
        Self {
            u,
            v,
            _padding: [0.; 62],
        }
    }
}
