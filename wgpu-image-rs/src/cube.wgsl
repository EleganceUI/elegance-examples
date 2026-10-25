// The cube: positions and colors per vertex, one matrix for the whole cube.

struct Uniforms {
    transform: mat4x4f,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Varyings {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
}

@vertex
fn vs_main(@location(0) position: vec3f, @location(1) color: vec3f) -> Varyings {
    var out: Varyings;
    out.position = uniforms.transform * vec4f(position, 1.0);
    out.color = color;
    return out;
}

// The image is gamma-encoded (like sRGB) and the colors already are: written as they are.
@fragment
fn fs_main(in: Varyings) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
