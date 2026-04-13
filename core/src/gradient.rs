//! Colors that transition progressively.
//!
//! This module defines a high-level, CPU-side representation of gradients.
//!
//! A [`Gradient`] describes a color field over a normalized coordinate space
//! (`0.0..=1.0`). It is independent of any specific geometry or transform.
//!
//! The gradient can later be converted into a GPU-friendly [`Packed`] form
//! (see `pack.rs`), where it becomes an affine coordinate system.
//!
//! ## Design Philosophy
//!
//! This abstraction separates:
//!
//! - **What** the gradient is (semantic)
//! - **How** it is evaluated (shader / affine transform)
//!
//! ## Coordinate Space
//!
//! `start_point` and `end_point` are defined in normalized space:
//!
//! ```text
//! (0,0) -------- (1,0)
//!   |              |
//!   |              |
//! (0,1) -------- (1,1)
//! ```
//!
//! This allows the same gradient to be reused across different geometries.
//!
//! ## Guarantees After `normalize()`
//!
//! - Offsets are clamped to `[0.0, 1.0]`
//! - Stops are sorted
//! - At most 8 stops
//! - At least 2 stops (duplicated if needed)
//!
//! These guarantees simplify GPU logic.

use crate::{Color, Point};
use std::cmp::Ordering;

/// The type of gradient.
///
/// This determines how the interpolation parameter `t`
/// is computed in shader code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientType {
    /// Linear interpolation along a direction.
    Linear,

    /// Radial interpolation from a center.
    Radial,

    /// Angular interpolation (circular gradient).
    Angular,

    /// Diamond-shaped interpolation (Manhattan distance).
    Diamond,
}

/// A single color stop in a gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    /// Position along gradient `[0.0, 1.0]`
    pub offset: f32,

    /// Color at this position
    pub color: Color,
}

/// High-level gradient definition.
///
/// This struct is CPU-friendly and flexible.
/// It should be normalized before packing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    /// Gradient evaluation type
    pub gradient_type: GradientType,

    /// Start point (normalized space)
    pub start_point: Point,

    /// End point (normalized space)
    pub end_point: Point,

    /// Ellipse ratio (for radial / angular / diamond)
    ///
    /// - `1.0` = circle
    /// - `< 1.0` = squashed
    /// - `> 1.0` = stretched
    pub aspect_ratio: f32,

    /// Up to 8 color stops
    pub stops: [Option<ColorStop>; 8],
}

impl Gradient {
    /// Create an empty gradient
    pub fn new(
        gradient_type: GradientType,
        start_point: Point,
        end_point: Point,
    ) -> Self {
        Self {
            gradient_type,
            start_point,
            end_point,
            aspect_ratio: 1.0,
            stops: [None; 8],
        }
    }

    /// Set ellipse aspect ratio
    pub fn with_aspect_ratio(mut self, ratio: f32) -> Self {
        self.aspect_ratio = ratio.max(0.0001);
        self
    }

    /// Add a stop
    pub fn add_stop(&mut self, stop: ColorStop) {
        if !stop.offset.is_finite() {
            return;
        }

        let mut stops: Vec<_> = self.stops.iter().flatten().copied().collect();

        stops.push(ColorStop {
            offset: stop.offset.clamp(0.0, 1.0),
            color: stop.color,
        });

        stops.sort_by(|a, b| {
            a.offset.partial_cmp(&b.offset).unwrap_or(Ordering::Equal)
        });

        stops.truncate(8);

        self.stops = [None; 8];
        for (i, s) in stops.into_iter().enumerate() {
            self.stops[i] = Some(s);
        }
    }

    /// Add stop by offset and color
    pub fn with_stop(mut self, offset: f32, color: Color) -> Self {
        self.add_stop(ColorStop { offset, color });
        self
    }

    /// Build Gradient With Stops
    pub fn with_stops(
        gradient_type: GradientType,
        start_point: Point,
        end_point: Point,
        stops: impl IntoIterator<Item = ColorStop>,
    ) -> Self {
        let mut g = Self::new(gradient_type, start_point, end_point);
        for s in stops {
            g.add_stop(s);
        }
        g.normalize();
        g
    }

    /// Normalize stops
    pub fn normalize(&mut self) {
        let mut stops: Vec<_> = self.stops.iter().flatten().copied().collect();

        if stops.is_empty() {
            return;
        }

        for s in &mut stops {
            s.offset = s.offset.clamp(0.0, 1.0);
        }

        stops.sort_by(|a, b| {
            a.offset.partial_cmp(&b.offset).unwrap_or(Ordering::Equal)
        });

        if stops.len() == 1 {
            let s = stops[0];
            stops.push(ColorStop {
                offset: 1.0,
                color: s.color,
            });
        }

        stops.truncate(8);

        self.stops = [None; 8];
        for (i, s) in stops.into_iter().enumerate() {
            self.stops[i] = Some(s);
        }
    }

    /// Scale alpha of all stops
    pub fn scale_alpha(mut self, factor: f32) -> Self {
        for s in self.stops.iter_mut().flatten() {
            s.color.a *= factor;
        }
        self
    }
}
