use lyon::path::{Path, PathEvent};
use lyon::math::{Point, Vector};
use lyon::path::iterator::PathIterator;
use iced_graphics::geometry::LineDash;
use iced_graphics::geometry::Path as IcedPath;
const FLAT_TOLERANCE: f32 = 0.05;

struct DashState<'a> {
    segments: &'a [f32],
    idx: usize,
    remaining: f32,
    draw: bool,
}

impl<'a> DashState<'a> {
    fn new(dash: LineDash<'a>) -> Self {
        let segments = dash.segments;
        assert!(!segments.is_empty());

        let idx = dash.offset % segments.len();

        Self {
            segments,
            idx,
            remaining: segments[idx],
            draw: idx % 2 == 0,
        }
    }

    fn advance(&mut self) {
        self.idx = (self.idx + 1) % self.segments.len();
        self.remaining = self.segments[self.idx];
        self.draw = !self.draw;
    }
}

pub fn dashed_path(
    path: &IcedPath,
    dash: LineDash<'_>,
) -> IcedPath {
    let tolerance = FLAT_TOLERANCE;
    let mut dash_state = DashState::new(dash);

    let mut builder = Path::builder();
    let mut drawing = false;

    for event in path.raw().iter().flattened(tolerance) {
        match event {
            PathEvent::Begin { at } => {
                // 不立即 begin，等 dash-on 再 begin
                // dash 是否 reset 取决于语义，这里保持连续
                let _ = at;
            }

            PathEvent::Line { from, to } => {
                dash_line(
                    from,
                    to,
                    &mut dash_state,
                    &mut builder,
                    &mut drawing,
                );
            }

            PathEvent::End { close, .. } => {
                if drawing {
                    builder.end(false);
                    drawing = false;
                }
            }

            _ => {}
        }
    }

    if drawing {
        builder.end(false);
    }

    IcedPath {
        raw: builder.build(),
        clip_offset: path.clip_offset,
        diff_path: path.diff_path.clone(),
    }
}

fn dash_line(
    from: Point,
    to: Point,
    state: &mut DashState<'_>,
    builder: &mut lyon::path::Builder,
    drawing: &mut bool,
) {
    let mut cursor = from;
    let mut remaining_len = (to - from).length();

    if remaining_len == 0.0 {
        return;
    }

    let dir = (to - from) / remaining_len;

    while remaining_len > 0.0 {
        let take = remaining_len.min(state.remaining);
        let next = cursor + dir * take;

        if state.draw {
            if !*drawing {
                let _ = builder.begin(cursor);
                *drawing = true;
            }
            let _ =builder.line_to(next);
        }

        cursor = next;
        remaining_len -= take;
        state.remaining -= take;

        if state.remaining <= 0.0 {
            if *drawing {
                builder.end(false);
                *drawing = false;
            }
            state.advance();
        }
    }
}