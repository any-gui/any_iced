use clipper2::Paths;
use lyon::path::Event;
use lyon::path::iterator::PathIterator;
use iced_graphics::geometry::{path, LineDash, Path};
use crate::core::{Point, Vector};
use crate::geometry::clip::{clip_by_path, ClipContour, ClipContourPoint};
use crate::geometry::dashed::dashed_path;

const FLAT_TOLERANCE: f32 = 0.05;

pub struct FlattenedPath {
    pub contours: Vec<ClipContour>
}

impl FlattenedPath {
    pub fn new() -> Self {
        Self { contours: vec![] }
    }

    pub fn push(&mut self, contour: ClipContour) {
        self.contours.push(contour)
    }

    pub fn clip(
        self,
        clip_path: Option<&FlattenedPath>,
        clip_offset: Option<f32>,
        diff_path: Option<&FlattenedPath>,
    ) -> FlattenedPath {
        let (close_paths,open_paths) = self.to_clipper_paths();
        let close = clip_by_path(close_paths,clip_path,clip_offset,diff_path,true);
        let open = clip_by_path(open_paths,clip_path,clip_offset,diff_path,false);
        FlattenedPath::merge(close, open)
    }

    // to clipper paths : include close paths and open paths
    pub fn to_clipper_paths(self) -> (Paths,Paths) {
        let Self { contours } = self;
        let mut close = Vec::new();
        let mut open = Vec::new();

        for c in contours {
            if c.closed {
                close.push(c.to_clipper_path());
            } else {
                open.push(c.to_clipper_path());
            }
        }
        (Paths::new(close), Paths::new(open))
    }

    pub fn merge(first: FlattenedPath, second: FlattenedPath) -> FlattenedPath {
        let mut contours = first.contours;
        contours.extend(second.contours);
        FlattenedPath { contours }
    }

    pub fn to_iced_path(&self) -> Path {
        let Self { contours } = self;
        Path::new(|builder| {
            for contour in contours {
                let ClipContour { points, closed } = contour;
                if points.len() > 1 {
                    for (index,p) in points.into_iter().enumerate() {
                        if index == 0 {
                            builder.move_to(Point::new(p.x,p.y))
                        } else {
                            builder.line_to(Point::new(p.x,p.y))
                        }
                    }
                    if *closed {
                        builder.close();
                    }
                }
            }
        } )
    }

    pub fn transform(self,vector: Vector) -> Self {
        let Self { contours } = self;
        let new_contours: Vec<ClipContour> = contours.into_iter().map(|c|c.transform(vector)).collect();
        FlattenedPath { contours: new_contours }
    }

    pub fn delta(&self,amount: f32) -> FlattenedPath {
        let Self { contours } = self;
        let new_contours: Vec<ClipContour> = contours.into_iter().flat_map(|c|c.delta(amount)).collect();
        FlattenedPath { contours: new_contours }
    }

    pub fn dashed(&self,dash: &LineDash<'_>,) -> FlattenedPath {
        let iced_path = self.to_iced_path();
        let path  = dashed_path(&iced_path,&dash);
        geometry_path_flatten(&path)
    }

    pub fn from_path(path: &Path) -> FlattenedPath {
        geometry_path_flatten(path)
    }
}

pub fn geometry_path_flatten(path: &Path) -> FlattenedPath {
    if path.flattened {
        lyon_path_flatten(path.raw(),true)
    } else {
        lyon_path_flatten(path.raw(),false)
    }
}

pub fn lyon_path_flatten(path: &lyon::path::Path,has_flattened: bool) -> FlattenedPath {
    let mut contours: FlattenedPath = FlattenedPath::new();
    let mut current_points: Vec<ClipContourPoint> = Vec::new();
    let mut current_closed: bool = false;
    if has_flattened {
        for event in path.iter() {
            match event {
                Event::Begin { at } => {
                    // 开始一个新的 sub-path
                    current_points.clear();
                    current_points.push(ClipContourPoint { x: at.x, y: at.y });
                    current_closed = false;
                }

                Event::Line { to, .. } => {
                    let p = ClipContourPoint { x: to.x, y: to.y };

                    // 去掉重复点
                    if current_points
                        .last()
                        .map_or(true, |last| last.x != p.x || last.y != p.y)
                    {
                        current_points.push(p);
                    }
                }

                Event::End { close, .. } => {
                    current_closed = close;

                    if close && current_points.len() >= 2 {
                        let first = current_points[0];
                        let last  = *current_points.last().unwrap();

                        if first.x == last.x && first.y == last.y {
                            let _ = current_points.pop();
                        }
                    }

                    if current_points.len() >= 2 {
                        contours.push(ClipContour {
                            points: current_points.clone(),
                            closed: current_closed,
                        });
                    }

                    current_points.clear();
                }

                // flattened 之后只会有 Begin / Line / End
                _ => {
                    unreachable!("Unhandled event: {:?}", event);
                }
            }
        }
    } else {
        for event in path.iter().flattened(FLAT_TOLERANCE) {
            match event {
                Event::Begin { at } => {
                    // 开始一个新的 sub-path
                    current_points.clear();
                    current_points.push(ClipContourPoint { x: at.x, y: at.y });
                    current_closed = false;
                }

                Event::Line { to, .. } => {
                    let p = ClipContourPoint { x: to.x, y: to.y };

                    // 去掉重复点
                    if current_points
                        .last()
                        .map_or(true, |last| last.x != p.x || last.y != p.y)
                    {
                        current_points.push(p);
                    }
                }

                Event::End { close, .. } => {
                    current_closed = close;

                    if close && current_points.len() >= 2 {
                        let first = current_points[0];
                        let last  = *current_points.last().unwrap();

                        if first.x == last.x && first.y == last.y {
                            let _ = current_points.pop();
                        }
                    }

                    if current_points.len() >= 2 {
                        contours.push(ClipContour {
                            points: current_points.clone(),
                            closed: current_closed,
                        });
                    }

                    current_points.clear();
                }

                // flattened 之后只会有 Begin / Line / End
                _ => {
                    unreachable!("Unhandled event: {:?}", event);
                }
            }
        }
    }

    contours
}