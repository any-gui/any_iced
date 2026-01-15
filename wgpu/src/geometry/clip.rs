use iced_graphics::geometry::{Fill, LineCap, LineJoin, Stroke};
use iced_graphics::geometry::fill::Rule;
use iced_graphics::Mesh;
use clipper2::{EndType, FillRule, JoinType, Path, Paths, Point as ClipPoint};
use lyon::math::Point as LyonPoint;
use lyon::path::Path as LyonPath;
use crate::geometry::coverage_aa::{build_aa_mesh, AA_FEATHER_ONE_SIDE};

const MITER_LIMIT: f64 = 4.0;
const SIMPLIFY_EPSILON: f64 = 0.05;

#[derive(Copy, Clone, Debug)]
pub struct ClipContourPoint {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl ClipContourPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }
    pub fn add(self, other: Self) -> Self { Self { x: self.x + other.x, y: self.y + other.y } }
    pub fn sub(self, other: Self) -> Self { Self { x: self.x - other.x, y: self.y - other.y } }
    pub fn mul(self, s: f32) -> Self { Self { x: self.x * s, y: self.y * s } }
    pub fn dot(self, other: Self) -> f32 { self.x * other.x + self.y * other.y }
    pub fn cross(self, other: Self) -> f32 { self.x * other.y - self.y * other.x }
    pub fn length(self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
    pub fn length_sq(self) -> f32 { self.x * self.x + self.y * self.y }
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { self } else { self.mul(1.0 / len) }
    }
    pub fn perp(self) -> Self {
        Self { x: -self.y, y: self.x }
    }
    pub fn rot90_cw(self) -> Self {
        Self { x: self.y, y: -self.x }
    }
    pub fn rot90_ccw(self) -> Self {
        Self { x: -self.y, y: self.x }
    }
    pub fn atan2(self) -> f32 {
        self.y.atan2(self.x)
    }
}

pub struct CoverageFillPath {
    pub fill_path: iced_graphics::geometry::Path,
    pub style: iced_graphics::geometry::fill::Style,
    pub aa_mesh: Mesh,
}

pub struct ClipContour {
    pub points: Vec<ClipContourPoint>,
    pub closed: bool,
}

impl ClipContour {
    pub fn to_coverage_stroke_path(
        self,
        stroke: &Stroke<'_>,
        scale_factor: f32,
    ) -> CoverageFillPath {
        let Self { points, closed } = self;
        let style = stroke.style;
        let mut path: Vec<ClipPoint> = points
            .into_iter()
            .map(|p| ClipPoint::new(p.x as f64, p.y as f64))
            .collect();
        // 3️⃣ 保证 CCW（offset > 0 向外）
        if signed_area(&path) < 0 {
            path.reverse();
        }
        let clipper_path = Path::new(path);

        let join_type = match stroke.line_join {
            LineJoin::Miter => JoinType::Miter,
            LineJoin::Round => JoinType::Round,
            LineJoin::Bevel => JoinType::Bevel,
        };

        let end_type = match (closed, stroke.line_cap) {
            (true, _) => EndType::Polygon,
            (false, LineCap::Butt) => EndType::Butt,
            (false, LineCap::Round) => EndType::Round,
            (false, LineCap::Square) => EndType::Square,
        };
        //aa + stroke path = width.
        let mut stroke_delta = (stroke.width - (AA_FEATHER_ONE_SIDE * 2. / scale_factor)) * 0.5;
        stroke_delta = stroke_delta.max(1. / scale_factor / 2.);
        let stroke = if closed {
            let stroke_outer = delta(&clipper_path, join_type, end_type, stroke_delta);
            let stroke_inner = delta(&clipper_path, join_type, end_type, -1. * stroke_delta);
            diff(&stroke_outer, stroke_inner)
        } else {
            delta(&clipper_path, join_type, end_type, stroke_delta)
        };

        CoverageFillPath {
            fill_path: build_lyon_path_from_paths(stroke.clone(), closed),
            style,
            aa_mesh: build_aa_mesh(stroke, style, scale_factor),
        }
    }

    pub fn to_coverage_fill_path(self, style: iced_graphics::geometry::fill::Style, scale_factor: f32) -> CoverageFillPath {
        let Self { points, closed } = self;
        let mut path: Vec<ClipPoint> = points
            .into_iter()
            .map(|p| ClipPoint::new(p.x as f64, p.y as f64))
            .collect();
        // 3️⃣ 保证 CCW（offset > 0 向外）
        if signed_area(&path) < 0 {
            path.reverse();
        }
        let fill_path = Path::new(path);
        let aa_offset = AA_FEATHER_ONE_SIDE * (1. / scale_factor);
        let fill_paths: Paths = delta(
            &fill_path,
            JoinType::Miter,
            EndType::Polygon,
            -1. * aa_offset,
        );

        CoverageFillPath {
            fill_path: build_lyon_path_from_paths(fill_paths.clone(), closed),
            style,
            aa_mesh: build_aa_mesh(fill_paths, style, scale_factor),
        }
    }
}

pub fn clip_by_path(
    contours: Vec<ClipContour>,
    clip_path: Vec<ClipContour>,
    
) -> Vec<ClipContour> {
    let mut results: Vec<ClipContour> = vec![];
    let clip_paths: Vec<Path> = clip_path.into_iter().map(|c|contour_to_clip_path(c)).collect();
    let clip_paths = Paths::new(clip_paths);
    for contour in contours {
        let closed = contour.closed;
        let cp = contour_to_clip_path(contour);
        let clipped = union(&Paths::new(vec![cp]), clip_paths.clone());
        results.extend(clip_path_to_contour(clipped, closed));
    }
    results
}

fn contour_to_clip_path(contour: ClipContour) -> Path {
    contour.points.into_iter().map(|p| ClipPoint::new(p.x as f64, p.y as f64)).collect()
}

fn clip_path_to_contour(paths: Paths,is_closed: bool) -> Vec<ClipContour> {
    paths.into_iter().map(|path|{
        ClipContour {
            points: path.into_iter().map(|point|ClipContourPoint::new(point.x() as f32, point.y() as f32)).collect(),
            closed: is_closed,
        }
    }).collect()
}

pub fn signed_area(path: &[ClipPoint]) -> i64 {
    let n = path.len();
    if n < 3 {
        return 0;
    }
    let mut area: i128 = 0;
    for i in 0..n {
        let p = &path[i];
        let q = &path[(i + 1) % n];

        area += (p.x() as i128) * (q.y() as i128) - (q.x() as i128) * (p.y() as i128);
    }
    area as i64
}

fn delta(path: &Path, join_type: JoinType, end_type: EndType, _delta: f32) -> Paths {
    let delta = _delta as f64;
    path.inflate(delta, join_type, end_type, MITER_LIMIT)
        .simplify(SIMPLIFY_EPSILON, false)
}

fn diff(subject: &Paths, clip: Paths) -> Paths {
    subject
        .to_clipper_subject()
        .add_clip(clip)
        .difference(FillRule::NonZero)
        .expect("clipper difference failed")
}

fn union(subject: &Paths, clip: Paths) -> Paths {
    subject
        .to_clipper_subject()
        .add_clip(clip)
        .union(FillRule::NonZero)
        .expect("clipper difference failed")
}

fn build_lyon_path_from_paths(outer_paths: Paths, is_closed: bool) -> iced_graphics::geometry::Path {
    let mut builder = LyonPath::builder();

    for path in outer_paths {
        if path.len() < 3 {
            continue;
        }

        // Clipper2 使用方向区分洞 / 外轮廓
        let area = signed_area(path.iter().as_slice());
        // lyon 不要求方向一致，但我们通常保持：
        let iter: Box<dyn Iterator<Item = &ClipPoint>> = if area < 0 {
            // hole → reverse to CCW
            Box::new(path.iter().rev())
        } else {
            Box::new(path.iter())
        };

        let mut it = iter.peekable();
        let first = it.next().unwrap();

        let _ = builder.begin(LyonPoint::new(first.x() as f32, first.y() as f32));

        for p in it {
            let _ = builder.line_to(LyonPoint::new(p.x() as f32, p.y() as f32));
        }

        if is_closed {
            builder.close();
        }
    }

    iced_graphics::geometry::Path {
        raw: builder.build(),
    }
}