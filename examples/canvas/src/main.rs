use iced::mouse;
use iced::widget::canvas::{
    self, Cache, Canvas, Geometry, Path, Stroke, stroke,
};
use iced::widget::center;
use iced::window;
use iced::{
    border, Color, Element, Length, Point, Rectangle, Renderer, Size,
    Subscription, Theme, Task
};

pub fn main() -> iced::Result {
    iced::daemon(CustomCanvas::new, CustomCanvas::update, CustomCanvas::view)
        .subscription(CustomCanvas::subscription)
        .title(CustomCanvas::title)
        .theme(Theme::Dark)
        .run()
}

struct CustomCanvas {
    window_id: Option<window::Id>,
    scale_factor: Option<f32>,
    cache: Cache,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    ScaleFactorReceived(f32),
}

impl CustomCanvas {
    fn new() -> (Self, Task<Message>) {
        let (_, open) = window::open(window::Settings::default());

        (
            Self {
                window_id: None,
                scale_factor: None,
                cache: Cache::default(),
            },
            open.map(Message::WindowOpened),
        )
    }

    fn title(&self, window: window::Id) -> String {
        "Custom Canvas".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                window::scale_factor(id).map(Message::ScaleFactorReceived)
            }
            Message::WindowClosed(id) => {
                if self.window_id == Some(id) {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::ScaleFactorReceived(scale) => {
                self.scale_factor = Some(scale);
                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        center(
            Canvas::new(self)
                .width(Length::Fixed(200.0))
                .height(Length::Fixed(200.0))
        ).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events()
            .map(Message::WindowClosed)
    }
}

impl<Message> canvas::Program<Message> for CustomCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw_with_bounds(
            renderer,
            bounds,
            |frame| {
                // 150x150 的圆角矩形，居中在 200x200 的 canvas 中
                // top_left = (200 - 150) / 2 = 25
                let rect_size = Size::new(150.0, 150.0);
                let top_left = Point::new(25.0, 25.0);

                // 圆角半径（可以根据需要调整）
                let radius = border::Radius::from(10.0);

                // 创建圆角矩形路径
                let rounded_rect = Path::rounded_rectangle(top_left, rect_size, radius);

                // 填充灰白色
                let fill_color = Color::from_rgb(0.4, 0.4, 0.4);
                frame.fill(&rounded_rect, fill_color);

                // 绘制 4px 白色边框
                frame.stroke(
                    &rounded_rect,
                    Stroke {
                        style: stroke::Style::Solid(Color::WHITE),
                        width: 4.0,
                        ..Stroke::default()
                    },
                );
            });
        vec![geometry]
    }
}
