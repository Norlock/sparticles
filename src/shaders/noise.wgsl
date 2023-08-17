fn rand(c: vec2<f32>) -> f32 {
	return fract(sin(dot(c.xy, vec2<f32>(12.9898,78.233))) * 43758.5453);
}

fn rf(n: f32) -> f32 {
 	return fract(cos(n*89.42)*343.42);
}

fn rv(n: vec2<f32>) -> vec2<f32> {
 	return vec2<f32>(rf(n.x * 23.62 - 300.0 + n.y * 34.35), rf(n.x * 45.13 + 256.0 + n.y * 38.89)); 
}

fn worley(n: vec2<f32>, s: f32) -> f32 {
    var dis = 2.0;

    for(var x = -1; x <= 1; x++) {
        for(var y = -1; y <= 1; y++) {
            let p = floor(n / s) + vec2<f32>(f32(x), f32(y));
            let d = length(rv(p) + vec2<f32>(f32(x), f32(y)) - fract(n / s));

            dis = min(dis, d);
        }
    }

    return 1.0 - dis;
}

// copy from https://www.shadertoy.com/view/4sc3z2


fn hash33(p3: vec3<f32>) -> vec3<f32> {
    var p3 = p3;
    let MOD3 = vec3<f32>(.1031,.11369,.13787);
	p3 = fract(p3 * MOD3);
    p3 += dot(p3, p3.yxz+19.19);
    return -1.0 + 2.0 * fract(vec3((p3.x + p3.y) * p3.z, (p3.x + p3.z) * p3.y, (p3.y + p3.z) * p3.x));
}

fn perlin_noise(p: vec3<f32>) -> f32 {
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
                	w.z),
        		mix(
                    mix(dot(pf - vec3<f32>(0., 1., 0.), hash33(pi + vec3<f32>(0., 1., 0.))), 
                        dot(pf - vec3<f32>(1., 1., 0.), hash33(pi + vec3<f32>(1., 1., 0.))),
                       	w.x),
                   	mix(dot(pf - vec3<f32>(0., 1., 1.), hash33(pi + vec3<f32>(0., 1., 1.))), 
                        dot(pf - vec3<f32>(1., 1., 1.), hash33(pi + vec3<f32>(1., 1., 1.))),
                       	w.x),
                	w.z),
    			w.y);
}

fn create_layers(v_pos: vec2<f32>, idx: f32, time: f32) -> vec3<f32> {
    var idx = idx;
    var sum = vec3<f32>(0.0);
    var amp = 1.;
    var scale = 2.;

    for (var i = 0; i < 5; i++) {
        let rotation = pitch_matrix(time * 0.1) * roll_matrix(time * -0.05);
        var noise = perlin_noise(rotation * vec3<f32>(v_pos.xy * scale, idx + time * 0.04));

        sum += vec3<f32>(noise) * amp;
        amp *= 0.9;
        scale *= 1.8;
    }

    return sum;
}
