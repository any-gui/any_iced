// =========================
// Vertex Input
// =========================

struct GradientVertexInput {
    @builtin(vertex_index) vertex_index: u32,

    @location(0) @interpolate(flat) colors_1: vec4<u32>,
    @location(1) @interpolate(flat) colors_2: vec4<u32>,
    @location(2) @interpolate(flat) colors_3: vec4<u32>,
    @location(3) @interpolate(flat) colors_4: vec4<u32>,

    @location(4) @interpolate(flat) offsets: vec4<u32>,

    // ✅ 新仿射坐标系
    @location(5) origin: vec2<f32>,
    @location(6) axis_x: vec2<f32>,
    @location(7) axis_y: vec2<f32>,

    @location(8) @interpolate(flat) gradient_type: u32,

    @location(9) position_and_scale: vec4<f32>,

    @location(10) border_color: vec4<f32>,
    @location(11) border_radius: vec4<f32>,
    @location(12) border_width: f32,

    @location(13) snap: u32,
};

// =========================
// Vertex Output
// =========================

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,

    @location(1) @interpolate(flat) colors_1: vec4<u32>,
    @location(2) @interpolate(flat) colors_2: vec4<u32>,
    @location(3) @interpolate(flat) colors_3: vec4<u32>,
    @location(4) @interpolate(flat) colors_4: vec4<u32>,

    @location(5) @interpolate(flat) offsets: vec4<u32>,

    @location(6) origin: vec2<f32>,
    @location(7) axis_x: vec2<f32>,
    @location(8) axis_y: vec2<f32>,

    @location(9) @interpolate(flat) gradient_type: u32,

    @location(10) position_and_scale: vec4<f32>,

    @location(11) border_color: vec4<f32>,
    @location(12) border_radius: vec4<f32>,
    @location(13) border_width: f32,
};
// =========================
// Vertex Shader
// =========================

@vertex
fn gradient_vs_main(input: GradientVertexInput) -> GradientVertexOutput {
    var out: GradientVertexOutput;

    var pos = input.position_and_scale.xy * globals.scale;
    var scale = input.position_and_scale.zw * globals.scale;

    var pos_snap = vec2<f32>(0.0);
    var scale_snap = vec2<f32>(0.0);

    if (bool(input.snap)) {
        pos_snap = round(pos + vec2(0.001)) - pos;
        scale_snap = round(pos + scale + vec2(0.001)) - pos - pos_snap - scale;
    }

    let min_radius = min(input.position_and_scale.z, input.position_and_scale.w) * 0.5;

    let border_radius = vec4<f32>(
        min(input.border_radius.x, min_radius),
        min(input.border_radius.y, min_radius),
        min(input.border_radius.z, min_radius),
        min(input.border_radius.w, min_radius)
    ) * globals.scale;

    let transform = mat4x4<f32>(
        vec4(scale.x + scale_snap.x + 1.0, 0.0, 0.0, 0.0),
        vec4(0.0, scale.y + scale_snap.y + 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(pos + pos_snap - vec2(0.5), 0.0, 1.0)
    );

    out.position =
        globals.transform *
        transform *
        vec4<f32>(vertex_position(input.vertex_index), 0.0, 1.0);

    out.colors_1 = input.colors_1;
    out.colors_2 = input.colors_2;
    out.colors_3 = input.colors_3;
    out.colors_4 = input.colors_4;

    out.offsets = input.offsets;

    out.origin = input.origin * globals.scale;
    out.axis_x = input.axis_x * globals.scale;
    out.axis_y = input.axis_y * globals.scale;

    out.gradient_type = input.gradient_type;

    out.position_and_scale = vec4(pos + pos_snap, scale + scale_snap);

    out.border_color = premultiply(input.border_color);
    out.border_radius = border_radius;
    out.border_width = input.border_width * globals.scale;

    return out;
}

// =========================
// Fragment Shader
// =========================

@fragment
fn gradient_fs_main(input: GradientVertexOutput) -> @location(0) vec4<f32> {

    let colors = array<vec4<f32>, 8>(
        unpack_color(input.colors_1.xy),
        unpack_color(input.colors_1.zw),
        unpack_color(input.colors_2.xy),
        unpack_color(input.colors_2.zw),
        unpack_color(input.colors_3.xy),
        unpack_color(input.colors_3.zw),
        unpack_color(input.colors_4.xy),
        unpack_color(input.colors_4.zw),
    );

    let offsets_1 = unpack_u32(input.offsets.xy);
    let offsets_2 = unpack_u32(input.offsets.zw);

    let offsets = array<f32, 8>(
        offsets_1.x, offsets_1.y, offsets_1.z, offsets_1.w,
        offsets_2.x, offsets_2.y, offsets_2.z, offsets_2.w,
    );

    let world = input.position.xy;

    var color = gradient(
        world,
        input.origin,
        input.axis_x,
        input.axis_y,
        input.gradient_type,
        colors,
        offsets
    );

    // 圆角裁剪
    let pos = input.position_and_scale.xy;
    let scale = input.position_and_scale.zw;

    let dist = rounded_box_sdf(
        -(world - pos - scale / 2.0) * 2.0,
        scale,
        input.border_radius * 2.0
    ) / 2.0;

    if (input.border_width > 0.0) {
        color = mix(
            color,
            input.border_color,
            clamp(0.5 + dist + input.border_width, 0.0, 1.0)
        );
    }

    return color * clamp(0.5 - dist, 0.0, 1.0);
}