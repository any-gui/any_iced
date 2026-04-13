//! Build different kinds of 2D shapes.
pub mod arc;

mod builder;

#[doc(no_inline)]
pub use arc::Arc;
pub use builder::Builder;

pub use lyon_path;
use lyon_path::PathEvent;
use iced_core::Rectangle;
use crate::core::border;
use crate::core::{Point, Size};

/// An immutable set of points that may or may not be connected.
///
/// A single [`Path`] can represent different kinds of 2D shapes!
#[derive(Debug, Clone)]
pub struct Path {
    /// raw lyon path
    pub raw: lyon_path::Path,
    /// Offset About Clip Path In Frame
    pub flattened: bool,
}

impl Path {
    /// Creates a new [`Path`] with the provided closure.
    ///
    /// Use the [`Builder`] to configure your [`Path`].
    pub fn new(f: impl FnOnce(&mut Builder)) -> Self {
        let mut builder = Builder::new();

        // TODO: Make it pure instead of side-effect-based (?)
        f(&mut builder);
        builder.build()
    }

    /// Creates a new [`Path`] representing a line segment given its starting
    /// and end points.
    pub fn line(from: Point, to: Point) -> Self {
        Self::new(|p| {
            p.move_to(from);
            p.line_to(to);
        })
    }

    /// Creates a new [`Path`] representing a rectangle given its top-left
    /// corner coordinate and its `Size`.
    pub fn rectangle(top_left: Point, size: Size) -> Self {
        Self::new(|p| p.rectangle(top_left, size))
    }

    /// Creates a new [`Path`] representing a rounded rectangle given its top-left
    /// corner coordinate, its [`Size`] and [`border::Radius`].
    pub fn rounded_rectangle(
        top_left: Point,
        size: Size,
        radius: border::Radius,
    ) -> Self {
        Self::new(|p| p.rounded_rectangle(top_left, size, radius))
    }

    /// Creates a new [`Path`] representing a circle given its center
    /// coordinate and its radius.
    pub fn circle(center: Point, radius: f32) -> Self {
        Self::new(|p| {
            p.circle(center, radius);
            p.close();
        })
    }

    /// Returns the internal [`lyon_path::Path`].
    #[inline]
    pub fn raw(&self) -> &lyon_path::Path {
        &self.raw
    }

    /// To lyon Path
    pub fn to_raw(self) -> lyon_path::Path {
        self.raw
    }

    /// Returns the current [`Path`] with the given transform applied to it.
    #[inline]
    pub fn transform(&self, transform: &lyon_path::math::Transform) -> Path {
        Path {
            raw: self.raw.clone().transformed(transform),
            flattened: self.flattened,
        }
    }
    
    /// Set flattened
    pub fn with_flattened(self, flattened: bool) -> Self {
        Self { raw: self.raw, flattened }
    }

    ///Get Bounding Box. For Bézier This is not precise
    pub fn get_bounding_rect(&self) -> Rectangle {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        let mut update = |p: lyon_path::math::Point| {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        };

        for event in self.raw.iter() {
            match event {
                PathEvent::Begin { at } => {
                    update(at);
                }
                PathEvent::Line { from, to } => {
                    update(from);
                    update(to);
                }
                PathEvent::Quadratic { from, ctrl, to } => {
                    update(from);
                    update(ctrl);
                    update(to);
                }
                PathEvent::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                } => {
                    update(from);
                    update(ctrl1);
                    update(ctrl2);
                    update(to);
                }
                PathEvent::End { last, first, .. } => {
                    update(last);
                    update(first);
                }
            }
        }

        // 处理空路径
        if min_x == f32::INFINITY {
            return Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            };
        }

        Rectangle {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}
