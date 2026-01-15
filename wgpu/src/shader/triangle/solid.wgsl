struct SolidVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) coverage: f32,
}

struct SolidVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) coverage: f32,
}

@vertex
fn solid_vs_main(input: SolidVertexInput) -> SolidVertexOutput {
    var out: SolidVertexOutput;

    out.color = premultiply(input.color);
    out.position = globals.transform * vec4<f32>(input.position, 0.0, 1.0);
    out.coverage = input.coverage;

    return out;
}

@fragment
fn solid_fs_main(input: SolidVertexOutput) -> @location(0) vec4<f32> {
    let alpha = input.color.a * input.coverage;
    let rgb = input.color.rgb * input.coverage;
    return vec4<f32>(rgb, alpha);
}
