struct Camera {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: Camera;

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,

    // InstanceRaw descompuesto
    @location(2) model_row_0: vec4<f32>,
    @location(3) model_row_1: vec4<f32>,
    @location(4) model_row_2: vec4<f32>,
    @location(5) model_row_3: vec4<f32>,
    @location(6) color: vec3<f32>,
) -> VSOutput {

    let model = mat4x4<f32>(
        model_row_0,
        model_row_1,
        model_row_2,
        model_row_3,
    );

    var out: VSOutput;
    out.position = camera.view_proj * model * vec4(position, 1.0);
    out.color = color;
    return out;
}
