struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VSInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,

    @location(2) instance_pos: vec3<f32>,
    @location(3) instance_color: vec3<f32>,
};

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VSInput) -> VSOutput {
    var out: VSOutput;
    let world_pos = in.position + in.instance_pos;
    out.position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = in.instance_color;
    return out;
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
