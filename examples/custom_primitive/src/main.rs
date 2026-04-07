use iced::time::Instant;
use iced::{wgpu, Background, Rectangle, Renderer, Theme};
use iced::widget::{center, container, column, row, shader, slider, text, canvas};
use iced::window;
use iced::{Center, Color, Element, Fill, Subscription};
use iced::advanced::renderer;
use crate::scene::Scene;

mod scene;


fn main() -> iced::Result {
    iced::application(IcedCubes::default, IcedCubes::update, IcedCubes::view)
        .subscription(IcedCubes::subscription)
        .run()
}

struct IcedCubes {
    scene: scene::Scene,
}

#[derive(Debug, Clone)]
enum Message {
    CubeAmountChanged(u32),
    CubeSizeChanged(f32),
    Tick(Instant),
    ShowDepthBuffer(bool),
    LightColorChanged(Color),
}

impl IcedCubes {
    fn new() -> Self {
        Self {
            scene: Scene {},
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CubeAmountChanged(amount) => {

            }
            Message::CubeSizeChanged(size) => {

            }
            Message::Tick(time) => {

            }
            Message::ShowDepthBuffer(show) => {

            }
            Message::LightColorChanged(color) => {

            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let container = container(
            "this is a container"
        ).style(
            |s| {
                container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.8))),
                    text_color: Some(Color::WHITE),
                    border: iced::Border {
                        radius: 24.0.into(),
                        width: 1.0,
                        color: Color::WHITE,
                    },
                    shadow: iced::Shadow::default(),
                    snap: false,
                }
            }
        ).center(200);
        let shader = shader(&self.scene).width(200).height(200);
        let rect_canvas = RectangleCanvas {};
        let canvas = canvas::Canvas::new(rect_canvas)
            .width(300)
            .height(300);
        center(column![shader, container, canvas].align_x(Center)).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }
}

impl Default for IcedCubes {
    fn default() -> Self {
        Self::new()
    }
}

struct RectangleCanvas;

impl canvas::Program<Message> for RectangleCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        renderer_style: &renderer::Style,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // 创建矩形路径
        let rect = canvas::Path::rectangle(
            iced::Point::new(50.0, 50.0),
            iced::Size::new(200.0, 100.0),
        );

        // 填充矩形
        frame.fill(&rect, Color::from_rgb(0.2, 0.6, 0.9));

        vec![frame.into_geometry()]
    }
}
