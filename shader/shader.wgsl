struct Uniforms {
    rect: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    // let quad = array<vec2<f32>, 6>(
    //     uniforms.rect.xy,
    //     uniforms.rect.zy,
    //     uniforms.rect.xw,
    //     uniforms.rect.zy,
    //     uniforms.rect.zw,
    //     uniforms.rect.xw,
    // );

    // var out: VertexOutput;
    // out.tex_coords = vec2<f32>(0.0);
    // out.tex_coords.x = select(0.0, 2.0, in_vertex_index == 1u);
    // out.tex_coords.y = select(0.0, 2.0, in_vertex_index == 2u);
    // out.clip_position = vec4<f32>(out.tex_coords * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 1.0, 1.0);
    // return out;

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    // out.clip_position = uniforms.rect * vec4<f32>(model.position, 1.0);
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
// @group(0) @binding(2)
// var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
