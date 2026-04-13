// =========================
// Vertex Input
// =========================

struct GradientVertexInput {
    @location(0) v_pos: vec2<f32>,

    @location(1) @interpolate(flat) colors_1: vec4<u32>,
    @location(2) @interpolate(flat) colors_2: vec4<u32>,
    @location(3) @interpolate(flat) colors_3: vec4<u32>,
    @location(4) @interpolate(flat) colors_4: vec4<u32>,

    @location(5) @interpolate(flat) offsets: vec4<u32>,

    // ✅ 新仿射坐标系
    @location(6) origin: vec2<f32>,
    @location(7) axis_x: vec2<f32>,
    @location(8) axis_y: vec2<f32>,

    @location(9) @interpolate(flat) gradient_type: u32,

    @location(10) coverage: f32,
};

// =========================
// Vertex Output
// =========================

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,

    @location(0) raw_position: vec2<f32>,

    @location(1) @interpolate(flat) colors_1: vec4<u32>,
    @location(2) @interpolate(flat) colors_2: vec4<u32>,
    @location(3) @interpolate(flat) colors_3: vec4<u32>,
    @location(4) @interpolate(flat) colors_4: vec4<u32>,

    @location(5) @interpolate(flat) offsets: vec4<u32>,

    @location(6) origin: vec2<f32>,
    @location(7) axis_x: vec2<f32>,
    @location(8) axis_y: vec2<f32>,

    @location(9) @interpolate(flat) gradient_type: u32,

    @location(10) coverage: f32,
};

// =========================
// Vertex Shader
// =========================

@vertex
fn gradient_vs_main(input: GradientVertexInput) -> GradientVertexOutput {
    var out: GradientVertexOutput;

    out.position = globals.transform * vec4<f32>(input.v_pos, 0.0, 1.0);
    out.raw_position = input.v_pos;

    out.colors_1 = input.colors_1;
    out.colors_2 = input.colors_2;
    out.colors_3 = input.colors_3;
    out.colors_4 = input.colors_4;

    out.offsets = input.offsets;

    out.origin = input.origin;
    out.axis_x = input.axis_x;
    out.axis_y = input.axis_y;

    out.gradient_type = input.gradient_type;
    out.coverage = input.coverage;

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

    // ✅ 关键修复：使用 world
    let world = input.position.xy;

    let color = gradient(
        world,
        input.origin,
        input.axis_x,
        input.axis_y,
        input.gradient_type,
        colors,
        offsets
    );

    let alpha = color.a * input.coverage;
    let rgb = color.rgb * input.coverage;

    return vec4<f32>(rgb, alpha);
}