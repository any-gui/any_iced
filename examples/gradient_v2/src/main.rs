use iced::time::Instant;
use iced::{Alignment, Background, Gradient, Length, Point, Rectangle, Renderer, Size, Theme};
use iced::widget::{center, container, column, shader, canvas, row, Canvas};
use iced::window;
use iced::{Center, Color, Element, Subscription};
use iced::advanced::graphics::text::align;
use iced::advanced::graphics::text::cosmic_text::Align;
use iced::advanced::renderer;
use iced::border::Radius;
use iced::gradient::{ColorStop, GradientType};

fn main() -> iced::Result {
    iced::application(GradientDemo::default, GradientDemo::update, GradientDemo::view)
        .subscription(GradientDemo::subscription)
        .run()
}

struct GradientDemo {
}

#[derive(Debug, Clone)]
enum Message {
    CubeAmountChanged(u32),
    CubeSizeChanged(f32),
    Tick(Instant),
    ShowDepthBuffer(bool),
    LightColorChanged(Color),
}

impl GradientDemo {
    fn new() -> Self {
        Self {
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
        let color_stops = [
            ColorStop {
                offset: 0.2,
                color: Color::from_rgb8(255, 216, 230),
            },
            ColorStop {
                offset: 0.6,
                color: Color::from_rgb8(167, 242, 255),
            },
            ColorStop {
                offset: 0.75,
                color: Color::from_rgb8(32, 100, 174),
            },
            ColorStop {
                offset: 0.9,
                color: Color::from_rgb8(138, 75, 104),
            },
        ];
        let linear_container = Gradient::with_stops(
            GradientType::Linear,
            Point::new(0.5, 0.0),
            Point::new(1.0, 1.0),
            color_stops,
        );
        let radial_container = Gradient::with_stops(
            GradientType::Radial,
            Point::new(0.25, 0.25),
            Point::new(0.5, 1.0),
            color_stops,
        );
        let angular_container = Gradient::with_stops(
            GradientType::Angular,
            Point::new(0.25, 0.75),
            Point::new(0.5, 1.0),
            color_stops,
        );
        let diamond_container = Gradient::with_stops(
            GradientType::Diamond,
            Point::new(0.75, 0.25),
            Point::new(0.5, 1.0),
            color_stops,
        );
        let linear_canvas = Gradient::with_stops(
            GradientType::Linear,
            Point::new(0.5, 0.0),
            Point::new(1.0, 1.0),
            color_stops,
        );
        let radial_canvas = Gradient::with_stops(
            GradientType::Radial,
            Point::new(0.5, 0.0),
            Point::new(1.0, 1.5),
            color_stops,
        );
        let angular_canvas = Gradient::with_stops(
            GradientType::Angular,
            Point::new(0.75, 0.25),
            Point::new(0.5, 0.0),
            color_stops,
        );
        let diamond_canvas = Gradient::with_stops(
            GradientType::Diamond,
            Point::new(0.0, 1.5),
            Point::new(1.0, 0.0),
            color_stops,
        );
        // 第一行：Container（占位/未来GPU）
        let top_row = row![
            gradient_container(linear_container),
            gradient_container(radial_container),
            gradient_container(angular_container),
            gradient_container(diamond_container),
        ]
            .height(Length::Fill).spacing(40.).align_y(Center);

        // 第二行：Canvas（当前CPU实现）
        let bottom_row = row![
            gradient_canvas(linear_canvas),
            gradient_canvas(radial_canvas),
            gradient_canvas(angular_canvas),
            gradient_canvas(diamond_canvas),
        ]
            .height(Length::Fill).spacing(40.).align_y(Center);

        column![top_row, bottom_row]
            .height(Length::Fill)
            .width(Length::Fill)
            .spacing(40.)
            .align_x(Alignment::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }
}

impl Default for GradientDemo {
    fn default() -> Self {
        Self::new()
    }
}

fn gradient_container<'a>(gradient: Gradient) -> iced::Element<'a, Message> {
    container("")
        .width(200.)
        .height(100.)
        .style(move |theme| {
            container::Style {
                background: Some(Background::Gradient(gradient)),
                text_color: Some(Color::WHITE),
                border: iced::Border {
                    radius: 24.0.into(),
                    width: 0.0,
                    color: Color::WHITE,
                },
                shadow: iced::Shadow::default(),
                snap: false,
            }
        })
        .into()
}

struct RectangleCanvas {
    gradient: Gradient,
}

impl RectangleCanvas {
    fn new(gradient: Gradient) -> Self {
        Self {
            gradient
        }
    }
}

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
        let rect = canvas::Path::rounded_rectangle(
            iced::Point::new(40.0, 80.0),
            Size::new(160.,80.),
            Radius::new(24)
        );

        // 填充矩形
        frame.fill(&rect, self.gradient);
        vec![frame.into_geometry()]
    }
}

fn gradient_canvas<'a>(gradient: Gradient) -> iced::Element<'a, Message> {
    Canvas::new(RectangleCanvas::new(gradient) )
        .width(200.)
        .height(200.)
        .into()
}
