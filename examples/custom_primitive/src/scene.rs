use crate::wgpu;

use iced::mouse;
use iced::time::Duration;
use iced::widget::shader::{self, Viewport};
use iced::{Color, Rectangle};

pub const MAX: u32 = 500;

#[derive(Clone)]
pub struct Scene {
}

impl Scene {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, time: Duration) {
    }
}

impl<Message> shader::Program<Message> for Scene {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new()
    }
}

/// A collection of `Cube`s that can be rendered.
#[derive(Debug)]
pub struct Primitive {
}

impl Primitive {
    pub fn new() -> Self {
        Self {}
    }
}

impl shader::Primitive for Primitive {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        // Upload data to GPU
        pipeline.update();
    }

    fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // Render primitive
        pipeline.render();
    }

    fn is_custom_primitive(&self) -> bool {
        true
    }

    /// Returns `true` if this [`iced::widget::shader::Primitive`] Sampler From Screen Texture.
    fn should_use_offscreen_texture(&self) -> bool {
        false
    }

    fn should_use_offscreen_layer(&self) -> bool {
        false
    }
}

impl shader::Pipeline for Pipeline {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Pipeline {
        Pipeline {

        }
    }
}

pub struct Pipeline {}

impl Pipeline {
    pub fn update(
        &mut self,
    ) {}

    pub fn render(&self) {}
}
