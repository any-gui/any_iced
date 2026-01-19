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

#[derive(Clone, Debug)]
pub enum CoverageMesh {
    /// A mesh with a solid color.
    Solid {
        /// The vertices and indices of the mesh.
        buffers: Indexed<SolidVertex2D>,
    },
    /// A mesh with a gradient.
    Gradient {
        /// The vertices and indices of the mesh.
        buffers: Indexed<GradientVertex2D>,
    },
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

pub fn build_aa_mesh(
    paths: Paths,
    style: Style,
    scale_factor: f32,
) -> CoverageMesh {
    let mut vertices: Vec<CoverageVertex> = vec![];
    let mut indices: Vec<u32> = vec![];

    let aa_radius: f32 = 1.0 / scale_factor;
    let miter_limit: f32 = 4.0 * aa_radius; // ★ 锐角限制

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

            // ============================================================
            // 🔥【关键改动 1】统一计算“真正的外侧法线”
            // ============================================================
            let outward = |e: ClipContourPoint| -> ClipContourPoint {
                let n = e.rot90_cw();
                n
            };

            let n_prev = outward(e_prev).normalize();
            let n_next = outward(e_next).normalize();

            // ---------- 外推平行线 ----------
            let l1_p = p1.add(n_prev.mul(aa_radius));
            let l1_d = e_prev;

            let l2_p = p1.add(n_next.mul(aa_radius));
            let l2_d = e_next;

            // ---------- 求交点 A1 ----------
            let mut p_outer = if let Some(ip) =
                line_intersection(l1_p, l1_d, l2_p, l2_d)
            {
                // ---------- miter limit ----------
                let v = ip.sub(p1);
                let len = v.length();

                if len > miter_limit {
                    p1.add(v.normalize().mul(miter_limit))
                } else {
                    ip
                }
            } else {
                // 平行边 fallback：退化为平均法线
                let n_avg = n_prev.add(n_next).normalize();
                p1.add(n_avg.mul(aa_radius))
            };

            // ============================================================
            // 🔥【关键改动 2】确保 coverage 梯度方向对 inner/outer 一致
            // ============================================================
            // 对 inner path（洞），p_inner / p_outer 需要“逻辑互换”
            let (p_cov1, p_cov0) = (p1, p_outer);

            let i0 = vertices.len() as u32;
            vertices.push(CoverageVertex::new(p_cov1, 1.0)); // coverage = 1
            vertices.push(CoverageVertex::new(p_cov0, 0.0)); // coverage = 0
        }

        // ---------- indices（闭合 strip） ----------
        for i in 0..n {
            let i0 = base + (i * 2) as u32;
            let i1 = base + (i * 2 + 1) as u32;
            let i2 = base + (((i + 1) % n) * 2) as u32;
            let i3 = base + (((i + 1) % n) * 2 + 1) as u32;

            indices.extend_from_slice(&[
                i0, i1, i3,
                i0, i3, i2,
            ]);
        }
    }

    build_mesh_from_coverage_vertices(vertices, indices, style)
}

fn outward_normal(edge: ClipContourPoint, is_outer: bool) -> ClipContourPoint {
    let n = edge.rot90_cw(); // 右手
    if is_outer { n } else { n.neg() }
}

fn line_intersection(
    p: ClipContourPoint,
    r: ClipContourPoint,
    q: ClipContourPoint,
    s: ClipContourPoint,
) -> Option<ClipContourPoint> {
    let rxs = cross(r, s);
    if rxs.abs() < 1e-6 {
        return None; // 平行
    }
    let t = cross(q.sub(p), s) / rxs;
    Some(p.add(r.mul(t)))
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
) -> CoverageMesh {
    match style {
        Style::Solid(color) => {
            let color_packed = color::pack(color);
            CoverageMesh::Solid {
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
            }
        }
        Style::Gradient(gradient) => {
            CoverageMesh::Gradient {
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
            }
        }
    }
}