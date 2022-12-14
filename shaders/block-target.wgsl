struct Camera {
    @location(0) position: vec4<f32>,
    @location(1) projection: mat4x4<f32>,
}

struct Target {
    @location(0) position: vec3<f32>,
    @location(1) face: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) target_position: vec3<f32>,
    @location(1) block_position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> tgt: Target;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.projection * vec4<f32>(model.position + tgt.position, 1.0);
    out.target_position = tgt.position;
    out.block_position = model.position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let xf = fract(in.block_position.x);
    let yf = fract(in.block_position.y);
    let zf = fract(in.block_position.z);

    if (in.target_position.y < 0.0) {
        return vec4<f32>(1.0, 1.0, 1.0, 0.0);
    } else if ((xf < 0.01 && yf < 0.01) || (xf < 0.01 && zf < 0.01) || (yf < 0.01 && zf < 0.01)) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 0.0);
    }
}
