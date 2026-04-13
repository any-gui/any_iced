//! GPU-friendly packed representation of gradients.
//!
//! This module converts [`Gradient`](crate::gradient::Gradient)
//! into a format directly consumable by WGSL shaders.
//!
//! ## Core Idea
//!
//! Instead of passing start/end points, we build a **2D affine coordinate system**:
//!
//! ```text
//! origin + axis_x + axis_y
//! ```
//!
//! This allows the shader to compute:
//!
//! ```text
//! p_local = inverse([axis_x axis_y]) * (p - origin)
//! ```
//!
//! Then gradient evaluation becomes trivial.
//!
//! ## Advantages
//!
//! - Supports all gradient types uniformly
//! - Enables elliptical gradients
//! - Works for both quad and mesh
//! - Matches Figma / Skia / WebRender design

use bytemuck::{Pod, Zeroable};
use half::f16;
use iced_core::{Color, Gradient, Point, Rectangle};
use iced_core::gradient::ColorStop;

/// Packed gradient for GPU
/// Packed gradient for GPU consumption.
///
/// This struct is a tightly packed, GPU-friendly representation of a [`Gradient`].
/// It is designed to:
///
/// - Be safely cast to raw bytes (`Pod`)
/// - Match WGSL memory layout exactly (`#[repr(C)]`)
/// - Minimize bandwidth (f16 packing)
/// - Encode a full 2D affine gradient space
///
/// ## Core Concept
///
/// Instead of storing `start/end` directly, we encode a **2D affine space**:
///
/// ```text
/// p_local = inverse([axis_x axis_y]) * (p - origin)
/// ```
///
/// Where:
///
/// - `origin` is the gradient origin (start point)
/// - `axis_x` is the primary gradient direction
/// - `axis_y` is the perpendicular axis (for ellipse support)
///
/// This allows all gradient types to share the same evaluation logic.
///
/// ## Memory Layout (C-compatible)
///
/// ```text
/// colors        64B
/// offsets       16B
/// origin         8B
/// axis_x         8B
/// axis_y         8B
/// gradient_type  4B
/// padding        4B
/// ----------------
/// total         112B (16-byte aligned)
/// ```
///
/// ## Shader Expectations
///
/// - Offsets are sorted ascending
/// - Offsets in `[0.0, 1.0]`
/// - Offsets > `1.0` are treated as invalid
/// - Colors beyond last valid stop are ignored
/// - `axis_x` and `axis_y` must not be degenerate
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Zeroable, Pod)]
pub struct Packed {
    /// Gradient colors (max 8 stops).
    ///
    /// Each entry stores one RGBA color using two `u32` values:
    ///
    /// ```text
    /// colors[i][0] = pack_f16(r, g)
    /// colors[i][1] = pack_f16(b, a)
    /// ```
    ///
    /// Each channel is stored as a 16-bit float (`f16`).
    ///
    /// Unused entries should contain transparent colors.
    pub colors: [[u32; 2]; 8],

    /// Gradient offsets (positions along the gradient).
    ///
    /// 8 offsets are packed into 4 `u32` values:
    ///
    /// ```text
    /// offsets[i] = pack_f16s(offset[2*i], offset[2*i+1])
    /// ```
    ///
    /// Shader-side expectations:
    ///
    /// - Must be sorted ascending
    /// - Range: `[0.0, 1.0]`
    /// - Values > `1.0` indicate unused stops
    pub offsets: [u32; 4],

    /// Gradient origin (start point).
    ///
    /// This is the reference point of the gradient in the same coordinate
    /// space as the rendered geometry.
    ///
    /// In shader:
    ///
    /// ```text
    /// d = p - origin
    /// ```
    ///
    /// All gradient evaluation is relative to this point.
    pub origin: [f32; 2],

    /// Primary gradient axis (X axis of gradient space).
    ///
    /// Typically:
    ///
    /// ```text
    /// axis_x = end_point - start_point
    /// ```
    ///
    /// This defines:
    ///
    /// - Gradient direction (linear)
    /// - Radius (radial)
    /// - Angle reference (angular)
    ///
    /// Must not be zero-length.
    pub axis_x: [f32; 2],

    /// Secondary gradient axis (Y axis of gradient space).
    ///
    /// This is perpendicular to `axis_x` and defines the second basis vector
    /// of the gradient coordinate system.
    ///
    /// Typically:
    ///
    /// ```text
    /// axis_y = perp(axis_x) * aspect_ratio
    /// ```
    ///
    /// Where:
    ///
    /// - `perp(v) = (-v.y, v.x)`
    /// - `aspect_ratio` controls ellipse distortion
    ///
    /// Used for:
    ///
    /// - Elliptical radial gradients
    /// - Angular gradients
    /// - Diamond gradients
    pub axis_y: [f32; 2],

    /// Gradient type discriminator.
    ///
    /// Controls how `t` is computed in shader.
    ///
    /// Values:
    ///
    /// ```text
    /// 0 = Linear
    /// 1 = Radial
    /// 2 = Angular
    /// 3 = Diamond
    /// 
    /// ```
    ///
    /// Shader uses this for branching.
    pub gradient_type: u32,
}

/// Pack Gradient → GPU format
pub fn pack(gradient: &Gradient, bounds: Rectangle) -> Packed {
    let mut colors = [[0u32; 2]; 8];
    let mut offsets_f16 = [f16::from_f32(0.0); 8];

    // =========================
    // 🎨 Colors + Offsets
    // =========================
    for (i, stop) in gradient.stops.iter().enumerate() {
        let stop = stop.unwrap_or(ColorStop {
            offset: 2.0,
            color: Color::TRANSPARENT,
        });

        colors[i] = [
            pack_f16s([
                f16::from_f32(stop.color.r),
                f16::from_f32(stop.color.g),
            ]),
            pack_f16s([
                f16::from_f32(stop.color.b),
                f16::from_f32(stop.color.a),
            ]),
        ];

        offsets_f16[i] = f16::from_f32(stop.offset);
    }

    let offsets = [
        pack_f16s([offsets_f16[0], offsets_f16[1]]),
        pack_f16s([offsets_f16[2], offsets_f16[3]]),
        pack_f16s([offsets_f16[4], offsets_f16[5]]),
        pack_f16s([offsets_f16[6], offsets_f16[7]]),
    ];

    // =========================
    // 🌍 normalized → world
    // =========================
    let start = Point {
        x: bounds.x + gradient.start_point.x * bounds.width,
        y: bounds.y + gradient.start_point.y * bounds.height,
    };

    let end = Point {
        x: bounds.x + gradient.end_point.x * bounds.width,
        y: bounds.y + gradient.end_point.y * bounds.height,
    };

    // =========================
    // 🔥 仿射坐标系（关键）
    // =========================

    // 主轴（渐变方向）
    let axis_x = [
        end.x - start.x,
        end.y - start.y,
    ];

    // 垂直轴（逆时针 90°）
    let perp = [
        -axis_x[1],
        axis_x[0],
    ];

    // 👉 椭圆控制（你设计的核心点）
    let axis_y = [
        perp[0] * gradient.aspect_ratio,
        perp[1] * gradient.aspect_ratio,
    ];

    // =========================
    // 📦 Packed
    // =========================
    Packed {
        colors,
        offsets,

        origin: [start.x, start.y],
        axis_x,
        axis_y,

        gradient_type: gradient.gradient_type as u32,
    }
}

/// Pack two f16 into u32
fn pack_f16s(f: [f16; 2]) -> u32 {
    ((f[0].to_bits() as u32) << 16) | (f[1].to_bits() as u32)
}
