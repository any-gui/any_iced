use iced::border::Radius;
use iced::widget::canvas::{
    self, Cache, Canvas, Geometry, LineDash, Path, Stroke, stroke,
};
use iced::widget::{center, Container};
use iced::window;
use iced::{
    Color, Element, Length, Point, Rectangle, Renderer, Subscription, Task,
    Theme,
};
use iced::{RendererStyle, Settings, Size, mouse};
use iced::widget::container::Style;
use iced_wgpu::geometry::flat::{geometry_path_flatten, FlattenedPath};
pub fn main() -> iced::Result {
    iced::daemon(CustomCanvas::new, CustomCanvas::update, CustomCanvas::view)
        .subscription(CustomCanvas::subscription)
        .title(CustomCanvas::title)
        .theme(Theme::Dark)
        .settings(Settings {
            antialiasing: false,
            ..Settings::default()
        })
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
    WindowResized,
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
                //println!("Window opened");
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
            Message::WindowResized => {
                //println!("Window resized");
                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        center(
            Container::new(
                Canvas::new(self)
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(200.0)),
            ).width(Length::Fixed(200.0))
                .height(Length::Fixed(200.0))
                .style(
                    |s| {
                        Style::default().background(Color::BLACK)
                    }
                )
        )
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_events().map(Message::WindowClosed),
            window::resize_events().map(|(id, size)| Message::WindowResized),
        ])
    }
}

impl<Message> canvas::Program<Message> for CustomCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        _style: &RendererStyle,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        //let clip_path = Path::circle(Point::new(100.0, 55.0), 50.);
        let clip_path = Path::new(|builder| {
            builder.move_to(Point::new(100., 10.));
            builder.line_to(Point::new(190., 100.));
            builder.line_to(Point::new(10., 100.));
            builder.close();
        });
        let border_path = Path::new(|builder| {
            builder.move_to(Point::new(100., 10.));
            builder.line_to(Point::new(190., 100.));
            builder.line_to(Point::new(10., 100.));
            builder.close();
        });
        let geometry = self.cache.draw_with_custom_config(
            renderer,
            bounds.size(),
            100.,
            |frame| {
                /*let path = Path::new(|builder| {

                    builder.move_to(Point::new(0.0, 41.7484));

                    // C
                    builder.bezier_curve_to(
                        Point::new(0.0, 18.69142),
                        Point::new(18.69142, 0.0),
                        Point::new(41.7484, 0.0),
                    );

                    // H
                    builder.line_to(Point::new(117.3304, 0.0));

                    // C
                    builder.bezier_curve_to(
                        Point::new(140.3876, 0.0),
                        Point::new(159.079, 18.69142),
                        Point::new(159.079, 41.7484),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(159.079, 60.7632),
                        Point::new(146.3668, 76.809),
                        Point::new(128.9774, 81.8506),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(128.864, 81.8836),
                        Point::new(128.7854, 81.9872),
                        Point::new(128.7854, 82.1052),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(128.7854, 82.2232),
                        Point::new(128.864, 82.327),
                        Point::new(128.9774, 82.3598),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(146.3668, 87.4014),
                        Point::new(159.079, 103.4472),
                        Point::new(159.079, 122.462),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(159.079, 145.519),
                        Point::new(140.3876, 164.2104),
                        Point::new(117.3304, 164.2104),
                    );

                    // H
                    builder.line_to(Point::new(41.7484, 164.2104));

                    // C
                    builder.bezier_curve_to(
                        Point::new(18.69142, 164.2104),
                        Point::new(0.0, 145.519),
                        Point::new(0.0, 122.462),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(0.0, 103.5714),
                        Point::new(12.54676, 87.611),
                        Point::new(29.7616, 82.46),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(29.9184, 82.413),
                        Point::new(30.027, 82.269),
                        Point::new(30.027, 82.1052),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(30.027, 81.9416),
                        Point::new(29.9184, 81.7974),
                        Point::new(29.7616, 81.7506),
                    );

                    // C
                    builder.bezier_curve_to(
                        Point::new(12.54676, 76.5994),
                        Point::new(0.0, 60.6392),
                        Point::new(0.0, 41.7484),
                    );

                    builder.close();
                });
                let flat_path = geometry_path_flatten(&path);
                let fg = Path::circle(Point::new(130.,100.), 100.);
                let fg_flat_path = geometry_path_flatten(&fg);
                let fg_path = fg_flat_path.clip(
                    Some(&flat_path),Some(-8.),None
                ).to_iced_path();
                let stroke_path = flat_path.delta(-4.).to_iced_path();
                let bg_path = flat_path.to_iced_path();
                frame.fill(
                    &bg_path,Color::from_rgb8(103,80,164)
                );
                frame.stroke(
                    &stroke_path,
                    Stroke {
                        style: stroke::Style::Solid(Color::BLACK.scale_alpha(0.2)),
                        width: 8.0,
                        line_dash: LineDash {
                            segments: &[],
                            offset: 0,
                        },
                        ..Default::default()
                    },
                );
                frame.fill(
                    &fg_path,Color::WHITE.scale_alpha(0.2)
                );*/
                let base_path = Path::rounded_rectangle(
                    Point::new(25., 25.),
                    Size::new(200., 200.),
                    Radius::new(4.),
                );
                let flat_base_path = FlattenedPath::from_path(&base_path);
                let dashed = FlattenedPath::from_path(&base_path).dashed(&LineDash{
                    segments: &[10.,10.],
                    offset: 0,
                    phase: 0.0,
                });
                let path = dashed.delta(2.);
                let path = path.clip(Some(&flat_base_path),Some(0.),None).to_iced_path();
                frame.fill(&base_path, Color::from_rgba8(255,255,255,0.2));
                //frame.fill(&path, Color::BLACK);
            },
            true,
            _style.scale_factor_for_aa,
        );
        vec![geometry]
    }
}
