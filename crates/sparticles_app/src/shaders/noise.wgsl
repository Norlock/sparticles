fn rand(c: vec2<f32>) -> f32 {
    return fract(sin(dot(c.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn rf(n: f32) -> f32 {
    return fract(cos(n * 89.42) * 343.42);
}

fn rv(n: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(rf(n.x * 23.62 - 300.0 + n.y * 34.35), rf(n.x * 45.13 + 256.0 + n.y * 38.89));
}

fn worley(n: vec2<f32>, s: f32) -> f32 {
    var dis = 2.0;

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let p = floor(n / s) + vec2<f32>(f32(x), f32(y));
            let d = length(rv(p) + vec2<f32>(f32(x), f32(y)) - fract(n / s));

            dis = min(dis, d);
        }
    }

    return 1.0 - dis;
}

// copy from https://www.shadertoy.com/view/4sc3z2


fn hash33(p3a: vec3<f32>) -> vec3<f32> {
    var p3 = p3a;
    let MOD3 = vec3<f32>(.1031, .11369, .13787);
    p3 = fract(p3 * MOD3);
    p3 += dot(p3, p3.yxz + 19.19);
    return -1.0 + 2.0 * fract(vec3((p3.x + p3.y) * p3.z, (p3.x + p3.z) * p3.y, (p3.y + p3.z) * p3.x));
}

fn perlin_noise_worley(p: vec3<f32>) -> f32 {
    let pi = floor(p);
    let pf = p - pi;

    let w = pf * pf * (3.0 - 2.0 * pf);

    return 	mix(
        mix(
            mix(dot(pf - vec3<f32>(0., 0., 0.), hash33(pi + vec3<f32>(0., 0., 0.))),
                dot(pf - vec3<f32>(1., 0., 0.), hash33(pi + vec3<f32>(1., 0., 0.))),
                w.x),
            mix(dot(pf - vec3<f32>(0., 0., 1.), hash33(pi + vec3<f32>(0., 0., 1.))),
                dot(pf - vec3<f32>(1., 0., 1.), hash33(pi + vec3<f32>(1., 0., 1.))),
                w.x),
            w.z
        ),
        mix(
            mix(dot(pf - vec3<f32>(0., 1., 0.), hash33(pi + vec3<f32>(0., 1., 0.))),
                dot(pf - vec3<f32>(1., 1., 0.), hash33(pi + vec3<f32>(1., 1., 0.))),
                w.x),
            mix(dot(pf - vec3<f32>(0., 1., 1.), hash33(pi + vec3<f32>(0., 1., 1.))),
                dot(pf - vec3<f32>(1., 1., 1.), hash33(pi + vec3<f32>(1., 1., 1.))),
                w.x),
            w.z
        ),
        w.y
    );
}

fn create_layers(v_pos: vec2<f32>, normal: f32, idxa: f32, time: f32) -> vec3<f32> {
    var idx = idxa;
    var sum = vec3<f32>(0.0);
    var amp = 1.;
    var scale = 2.;

    for (var i = 0; i < 5; i++) {
        let rotation = pitch_matrix(time * 0.1) * roll_matrix(time * -0.05);
        var noise = perlin_noise_worley(rotation * vec3<f32>(v_pos.xy * scale, idx + time * 0.04));

        sum += vec3<f32>(noise) * amp * normal;
        amp *= 0.9;
        scale *= 1.8;
    }

    return sum;
}


fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2(127.1, 311.7));

    return  -1. + 2. * fract(sin(h) * 43758.5453123);
}

fn hash22(pa: vec2<f32>) -> vec2<f32> {
    var p = pa * mat2x2<f32>(vec2(127.1, 311.7), vec2(269.5, 183.3));
    p = -1.0 + 2.0 * fract(sin(p) * 43758.5453123);
    //return sin(p * 6.283 + iTime); // todo time
    return sin(p * 6.283); // todo time
}

fn perlin_noise(p: vec2<f32>) -> f32 {
    var pi = floor(p);
    var pf = p - pi;

    var w = pf * pf * (3. - 2. * pf);

    var f00 = dot(hash22(pi + vec2(.0, .0)), pf - vec2(.0, .0));
    var f01 = dot(hash22(pi + vec2(.0, 1.)), pf - vec2(.0, 1.));
    var f10 = dot(hash22(pi + vec2(1.0, 0.)), pf - vec2(1.0, 0.));
    var f11 = dot(hash22(pi + vec2(1.0, 1.)), pf - vec2(1.0, 1.));

    var xm1 = mix(f00, f10, w.x);
    var xm2 = mix(f01, f11, w.x);
    var ym = mix(xm1, xm2, w.y);
    return ym;
}

fn noise_sum(pa: vec2<f32>) -> f32 {
    var p = pa * 4.;
    var a = 1.;
    var r = 0.;
    var s = 0.;

    for (var i = 0; i < 5; i++) {
        r += a * perlin_noise(p);
        s += a;
        p *= 2.;
        a *= .5;
    }

    return r / s;
}

fn noise_sum_abs(pa: vec2<f32>) -> f32 {
    var p = pa * 4.;
    var a = 1.;
    var r = 0.;
    var s = 0.;

    for (var i = 0; i < 5; i++) {
        r += a * abs(perlin_noise(p));
        s += a;
        p *= 2.;
        a *= .5;
    }

    return (r / s - .135) / (.06 * 3.);
}

fn noise_sum_abs_sin(pa: vec2<f32>) -> f32 {
    var p = pa * 7.0 / 4.0;
    var f = noise_sum_abs(p);
    f = sin(f * 1.5 + p.x * 4.0);

    return f * f;
}

fn noise_one_octave(p: vec2<f32>) -> f32 {
    var r = 0.0;
    r += 0.125 * abs(perlin_noise(p * 30.));
    return r;
}

fn noise(pa: vec2<f32>) -> f32 {
    var p = pa;
	//#ifdef marble
    p.x = noise_sum_abs_sin(p);
    //#elif defined turbulence
    p.y = noise_sum_abs(p);
    //#elif defined granite
    p.x = noise_one_octave(p);
    //#elif defined cloud
    return noise_sum(p);
    //#endif
}
