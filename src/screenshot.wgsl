struct PushConstants {
    window: vec4<f32>,
    p_math: vec4<f32>,
    p_gfx: vec4<f32>,
    screensize: vec2<f32>,
};

var<push_constant> pc: PushConstants;
@group(0) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;

fn square(z: vec2<f32>) -> vec2<f32> {
    return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
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
        let hue = angle(((f32(num) / pc.p_math.w)
                    * pc.p_gfx.x) - pc.p_gfx.y);

        let val = ((f32(num) / pc.p_math.w))
                * (abs(pc.p_gfx.w) - abs(pc.p_gfx.z))
            + abs(pc.p_gfx.z);

        return cubehelix(hue, 1.0, val);
    }
}

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let x = global_invocation_id.x % u32(pc.screensize.x);
    let y = global_invocation_id.x / u32(pc.screensize.x);

    let cx = pc.window.x + f32(x) * (pc.window.z - pc.window.x)
        / pc.screensize.x;
    let cy = pc.window.y + f32(y) * (pc.window.w - pc.window.y)
        / pc.screensize.y;

    textureStore(output, vec2(i32(x), i32(y)), color(iterations(vec2(cx, cy))))
}