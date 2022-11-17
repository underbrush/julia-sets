struct PushConstants {
    window: vec4<f32>,
    p_math: vec4<f32>,
    p_gfx: vec4<f32>,
};

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

var<push_constant> pc: PushConstants;

// (a + bi)^2 = (a^2 - b^2) + 2abi

fn square(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}

fn square_a(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, -2.0 * z.x * z.y);
}

fn iterations(
    start: vec2<f32>
) -> u32 {
    var z = start;
    for (var x: u32 = 0u; x < u32(pc.p_math.w); x = x + 1u) {
        if dot(z, z) > pc.p_math.z {
            return x;
        }
        z = square(z) + pc.p_math.xy;
    }
    return u32(pc.p_math.w);
}

fn iterations_a(
    start: vec2<f32>
) -> u32 {
    var z = start;
    for (var x: u32 = 0u; x < u32(pc.p_math.w); x = x + 1u) {
        if dot(z, z) > pc.p_math.z {
            return x;
        }
        z = square_a(z) + pc.p_math.xy;
    }
    return u32(pc.p_math.w);
}


fn iterations_m(
    c: vec2<f32>
) -> u32 {
    var z = pc.p_math.xy;
    for (var x: u32 = 0u; x < u32(pc.p_math.w); x = x + 1u) {
        if dot(z, z) > 4.0 {
            return x;
        }
        z = square(z) + c;
    }
    return u32(pc.p_math.w);
}

fn iterations_t(
    c: vec2<f32>
) -> u32 {
    var z = vec2<f32>(0.0, 0.0);
    for (var x: u32 = 0u; x < u32(pc.p_math.w); x = x + 1u) {
        if dot(z, z) > pc.p_math.z {
            return x;
        }
        z = square_a(z) + c;
    }
    return u32(pc.p_math.w);
}


fn HSL(h: f32, s: f32, l: f32) -> vec4<f32> {
    let a = s * l * (1.0 - l);
    let h_cos = cos(h);
    let h_sin = sin(h);

    var r = (l + a * (-0.14861 * h_cos + 1.78277 * h_sin));
    var g = (l + a * (-0.29227 * h_cos - 0.90649 * h_sin));
    var b = (l + a * (1.97294 * h_cos));

    r = min(max(0.0, pow(r, 2.2)), 1.0);
    g = min(max(0.0, pow(g, 2.2)), 1.0);
    b = min(max(0.0, pow(b, 2.2)), 1.0);

    return vec4(r, g, b, 1.0);
}

fn cubehelix(h: f32, s: f32, l: f32) -> vec4<f32> {
    // let pi = 3.14159265;
    let frac = 0.5 - (cos(h) / 2.0);

    let n = (0.3 + 0.4 * frac);
    let new_l = (2.0 - 4.0 * n) * (l * l) + (4.0 * n - 1.0) * l;

    return HSL(
        h,
        min(1.0, s * (0.5 + 0.75 * frac)),
        new_l
    );
}

fn angle(x: f32) -> f32 {
    let tau = 6.28318530718;
    return (tau  + (x % tau)) % tau;
}

fn color(num: u32) -> vec4<f32> {
    if num == u32(pc.p_math.w) {
        if pc.p_gfx.z < 0.0 || pc.p_math.w < 0.0 {
            return vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            return vec4(1.0, 1.0, 1.0, 1.0);
        }
    } else {
        let hue = angle(f32(num) * pc.p_gfx.y + pc.p_gfx.x);

//        let val = ((f32(num) / pc.p_math.w))
//                * (abs(pc.p_gfx.w) - abs(pc.p_gfx.z))
//            + abs(pc.p_gfx.z);

        let val = (1. - pow(0.99, f32(num)))
                * (abs(pc.p_gfx.w) - abs(pc.p_gfx.z))
                + abs(pc.p_gfx.z);

        return HSL(hue, 1.0, val);
    }
}

@vertex
fn vs_main(@builtin(vertex_index) VertexIndex : u32) -> VertexOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0)
    );

    var output : VertexOutput;
    output.position = vec4<f32>(
        (2.0 * pos[VertexIndex].x - 1.0),
        (1.0 - 2.0 * pos[VertexIndex].y),
        0.0, 1.0);
    output.tex_coord = pos[VertexIndex];
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = pc.window.x + (in.tex_coord.x) * (pc.window.z - pc.window.x);
    let y = pc.window.y + (in.tex_coord.y) * (pc.window.w - pc.window.y);

    let oversample = 1u;
    var iters = 0u;

    for (var dx: u32 = 0u; dx < oversample; dx++) {
        for (var dy: u32 = 0u; dy < oversample; dy++) {
            let x2 = x + f32(dx) * dpdx(in.tex_coord.x) * (pc.window.z - pc.window.x)/f32(oversample);
            let y2 = y + f32(dy) * dpdy(in.tex_coord.y) * (pc.window.w - pc.window.y)/f32(oversample);
            iters += iterations(vec2(x2, y2));
        }
    }

    iters = iters / (oversample * oversample);

    return color(iters);
}

