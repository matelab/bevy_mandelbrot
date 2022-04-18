struct MandelbrotFS {
    center: vec2<f32>;
    start: vec2<f32>;
    scale: f32;
    aspect: f32;
    iters: i32;
};

[[group(1), binding(0)]]
var<uniform> fs: MandelbrotFS;

[[stage(fragment)]]
fn fragment([[location(2)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
    var z: vec2<f32>;
    var i: i32;
    let iters = fs.iters;
    z = fs.start;
    var p: vec2<f32>;
    p = vec2<f32>((fs.aspect * (uv.x - 0.5)) / fs.scale + fs.center.x, (uv.y - 0.5) / fs.scale + fs.center.y);
    for (i = 0; i < iters; i = i + 1) {
        let x = (z.x * z.x - z.y * z.y) + p.x;
        let y = (2.0 * z.x * z.y) + p.y;

        if (((x * x) + (y * y)) > 4.0) {
            break;
        }
        z.x = x;
        z.y = y;
    }
    var col: f32;
    if (i == iters) {
        col = 0.0;
    } else {
        col = f32(i) / f32(iters);
    }
    return vec4<f32>(col, col, col, 1.0);

}
