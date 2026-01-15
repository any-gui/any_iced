use clipper2::{Paths};
use iced_graphics::{color, Mesh};
use iced_graphics::geometry::{Style};
use crate::geometry::clip::{signed_area, ClipContourPoint};
use clipper2::{Point as ClipPoint};
use iced_graphics::mesh::{GradientVertex2D, Indexed, SolidVertex2D};

pub const AA_FEATHER_ONE_SIDE: f32 = 0.5;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CoverageVertex {
    pub position: [f32; 2],
    pub coverage: f32,
}

impl CoverageVertex {
    #[inline]
    pub fn new(position: ClipContourPoint, coverage: f32) -> Self {
        debug_assert!(
            coverage == 0.0 || coverage == 1.0,
            "coverage must be 0 or 1"
        );

        CoverageVertex {
            position: [position.x, position.y],
            coverage,
        }
    }
}

pub fn build_aa_mesh(paths: Paths, style: Style, scale_factor: f32) -> Mesh {
    let mut vertices: Vec<CoverageVertex> = vec![];
    let mut indices: Vec<u32> = vec![];
    let aa_radius: f32 = 1. / scale_factor;
    for path in paths.into_iter() {
        let n = path.len();
        if n < 3 {
            continue;
        }

        let pts = path.iter().as_slice();
        let is_outer = signed_area(pts) > 0;

        let base = vertices.len() as u32;
        for i in 0..n {
            let p0 = to_vec2(&pts[(i + n - 1) % n]);
            let p1 = to_vec2(&pts[i]);
            let p2 = to_vec2(&pts[(i + 1) % n]);

            let e_prev = normalize(p1.sub(p0));
            let e_next = normalize(p2.sub(p1));
            // 角分线（bisector）
            let mut bisector = e_prev.add(e_next).normalize();
            // 法线旋转 90°得到外方向
            let normal = bisector.rot90_cw().normalize();
            if !is_outer {
                bisector = bisector.mul(-1.0);
            } // 洞反向

            // 凹角检测
            let turn = cross(e_prev, e_next);
            let is_concave = if is_outer { turn < 0.0 } else { turn > 0.0 };

            let aa_len = if is_concave { 0.0 } else { aa_radius };

            let p_inner = p1;
            let p_outer = p1.add(normal.mul(aa_len));
            let i0 = vertices.len() as u32;
            vertices.push(CoverageVertex::new(p_inner, 1.)); //1
            vertices.push(CoverageVertex::new(p_outer, 0.)); //0
        }
        // ---------- 生成 indices（闭合！） ----------
        for i in 0..n {
            let i0 = base + (i * 2) as u32;
            let i1 = base + ((i * 2 + 1) % (2 * n)) as u32;
            let i2 = base + (((i + 1) % n) * 2) as u32;
            let i3 = base + (((i + 1) % n) * 2 + 1) as u32;

            indices.extend_from_slice(&[i0, i1, i3, i0, i3, i2]);
        }
    }

    build_mesh_from_coverage_vertices(vertices, indices, style)
    /* `CoverageMesh` value */
}

fn cross(a: ClipContourPoint, b: ClipContourPoint) -> f32 {
    a.x * b.y - a.y * b.x
}

fn normalize(v: ClipContourPoint) -> ClipContourPoint {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len == 0.0 { v } else { v.mul(1.0 / len) }
}

#[inline]
fn to_vec2(p: &ClipPoint) -> ClipContourPoint {
    ClipContourPoint {
        x: p.x() as f32,
        y: p.y() as f32,
    }
}

fn build_mesh_from_coverage_vertices(
    vertices: Vec<CoverageVertex>,
    indices: Vec<u32>,
    style: Style,
) -> Mesh {
    match style {
        Style::Solid(color) => {
            let color_packed = color::pack(color);
            Mesh::Solid {
                buffers: Indexed {
                    vertices: vertices
                        .into_iter()
                        .map(|v| SolidVertex2D {
                            position: v.position,
                            color: color_packed,
                            coverage: v.coverage,
                        })
                        .collect(),
                    indices,
                },
                transformation: Default::default(),
                clip_bounds: Default::default(),
            }
        }
        Style::Gradient(gradient) => {
            Mesh::Gradient {
                buffers: Indexed {
                    vertices: vertices
                        .into_iter()
                        .map(|v| GradientVertex2D {
                            position: v.position,
                            gradient: gradient.pack(),
                            coverage: v.coverage,
                        })
                        .collect(),
                    indices,
                },
                transformation: Default::default(),
                clip_bounds: Default::default(),
            }
        }
    }
}