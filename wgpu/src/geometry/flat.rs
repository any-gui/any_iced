use lyon::path::Event;
use lyon::path::iterator::PathIterator;
use iced_graphics::geometry::Path;
use crate::geometry::clip::{ClipContour, ClipContourPoint};

const FLAT_TOLERANCE: f32 = 0.05;

pub fn geometry_path_flatten(path: &Path) -> Vec<ClipContour> {
    lyon_path_flatten(path.raw())
}

pub fn lyon_path_flatten(path: &lyon::path::Path) -> Vec<ClipContour> {
    let mut contours: Vec<ClipContour> = Vec::new();

    let mut current_points: Vec<ClipContourPoint> = Vec::new();
    let mut current_closed: bool = false;

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

    contours
}