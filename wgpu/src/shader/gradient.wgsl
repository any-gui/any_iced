// =========================
// gradient.wgsl (shared)
// =========================

const PI: f32 = 3.141592653589793;

const GRADIENT_LINEAR: u32 = 0u;
const GRADIENT_RADIAL: u32 = 1u;
const GRADIENT_ANGULAR: u32 = 2u;
const GRADIENT_DIAMOND: u32 = 3u;

// =========================
// 🎲 Stable dithering
// =========================
fn random(coords: vec2<f32>) -> f32 {
    return fract(sin(dot(coords, vec2(12.9898, 78.233))) * 43758.5453);
}

// =========================
// 🔍 find last valid stop
// =========================
fn find_last_index(offsets: array<f32, 8>) -> i32 {
    var last: i32 = 0;

    for (var i: i32 = 0; i < 8; i++) {
        if (offsets[i] > 1.0) {
            return max(i - 1, 0);
        }
        last = i;
    }

    return last;
}

// =========================
// 🧮 affine → uv（稳定版）
// =========================
fn compute_uv(
    p: vec2<f32>,
    origin: vec2<f32>,
    axis_x: vec2<f32>,
    axis_y: vec2<f32>
) -> vec2<f32> {

    let rel = p - origin;

    let u = dot(rel, axis_x) / dot(axis_x, axis_x);
    let v = dot(rel, axis_y) / dot(axis_y, axis_y);

    return vec2<f32>(u, v);
}

// =========================
// 🧮 compute t
// =========================
fn compute_t(
    p: vec2<f32>,
    origin: vec2<f32>,
    axis_x: vec2<f32>,
    axis_y: vec2<f32>,
    ttype: u32
) -> f32 {

    let uv = compute_uv(p, origin, axis_x, axis_y);
    let u = uv.x;
    let v = uv.y;

    if (ttype == GRADIENT_LINEAR) {
        return u;
    }

    if (ttype == GRADIENT_RADIAL) {
        return length(vec2<f32>(u, v));
    }

    if (ttype == GRADIENT_ANGULAR) {
        // 6点钟起点 + 顺时针
        let angle = atan2(u, v);
        var t = fract(angle / (2.0 * PI) + 1.0);
        return 1.0 - t;
    }

    if (ttype == GRADIENT_DIAMOND) {
        return abs(u) + abs(v);
    }

    return 0.0;
}

// =========================
// 🎨 gradient core
// =========================
fn gradient(
    p: vec2<f32>,
    origin: vec2<f32>,
    axis_x: vec2<f32>,
    axis_y: vec2<f32>,
    ttype: u32,
    colors: array<vec4<f32>, 8>,
    offsets: array<f32, 8>
) -> vec4<f32> {

    let last = find_last_index(offsets);
    let t = compute_t(p, origin, axis_x, axis_y, ttype);

    let noise_strength: f32 = 1.0 / 255.0;

    var color: vec4<f32>;

    // =========================
    // 🟩 非 Angular
    // =========================
    if (ttype != GRADIENT_ANGULAR) {

        if (t <= offsets[0]) {
            color = colors[0];
        } else if (t >= offsets[last]) {
            color = colors[last];
        } else {

            for (var i: i32 = 0; i < last; i++) {
                let o1 = offsets[i];
                let o2 = offsets[i + 1];

                if (o1 <= t && t <= o2) {

                    let span = o2 - o1;

                    if (abs(span) < 1e-5) {
                        color = colors[i + 1];
                    } else {
                        var f = (t - o1) / span;
                        f = clamp(f, 0.0, 1.0);
                        color = mix(colors[i], colors[i + 1], f);
                    }

                    break;
                }
            }
        }

        let n = random(floor(p * 0.5)) - 0.5;
        return vec4<f32>(color.rgb + vec3<f32>(n * noise_strength), color.a);
    }

    // =========================
    // 🔵 Angular（闭环）
    // =========================

    for (var i: i32 = 0; i < last; i++) {
        let o1 = offsets[i];
        let o2 = offsets[i + 1];

        if (o1 <= t && t <= o2) {

            let span = o2 - o1;

            if (abs(span) < 1e-5) {
                color = colors[i + 1];
            } else {
                var f = (t - o1) / span;
                f = clamp(f, 0.0, 1.0);
                color = mix(colors[i], colors[i + 1], f);
            }

            let n = random(floor(p * 0.5)) - 0.5;
            return vec4<f32>(color.rgb + vec3<f32>(n * noise_strength), color.a);
        }
    }

    let o_last = offsets[last];
    let o_first = offsets[0];

    let span = (1.0 - o_last) + o_first;

    if (abs(span) < 1e-5) {
        return colors[0];
    }

    var tw: f32;

    if (t >= o_last) {
        tw = (t - o_last) / span;
    } else {
        tw = (t + (1.0 - o_last)) / span;
    }

    var f = clamp(tw, 0.0, 1.0);
    color = mix(colors[last], colors[0], f);

    let n = random(floor(p * 0.5)) - 0.5;
    return vec4<f32>(color.rgb + vec3<f32>(n * noise_strength), color.a);
}