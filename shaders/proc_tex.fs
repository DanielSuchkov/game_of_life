#version 400

out vec4 gl_FragColor;
in vec2 gl_FragCoord;
uniform vec2 u_resolution;           // viewport resolution (in pixels)
uniform float u_time;           // shader playback time (in seconds)

// This is my favorite fire palette. It's trimmed down for shader usage, and is based on an
// article I read at Hugo Elias's site years ago. I'm sure most old people, like me, have
// visited his site at one time or another:
//
// http://freespace.virgin.net/hugo.elias/models/m_ffire.htm
//
vec3 firePalette(float i){

    float T = 1200. + 1600.*i; // Temperature range (in Kelvin).
    vec3 L = vec3(7.4, 5.6, 4.4); // Red, green, blue wavelengths (in hundreds of nanometers).
    L = pow(L,vec3(5.0)) * (exp(1.43876719683e5/(T*L))-1.0);
    return 1.0-exp(-5e8/L); // Exposure level. Set to 50. For 70, change the 5 to a 7, etc.
}

// Hash function. This particular one probably doesn't disperse things quite as nicely as some
// of the others around, but it's compact, and seems to work.
//
vec3 hash33(vec3 p){

    float n = sin(dot(p, vec3(7, 157, 113)));
    return fract(vec3(2097152, 262144, 32768)*n);
}

// 3D Voronoi: Obviously, this is just a rehash of IQ's original.
//
float voronoi(vec3 p){

    vec3 b, r, g = floor(p);
    p = fract(p); // p -= g; works on some GPUs, but not all, for some annoying reason.

    // Maximum value: I think outliers could get as high as 3, the squared diagonal length
    // of the unit cube, with the mid point being 0.75. Is that right? Either way, for this
    // example, the maximum is set to one, which would cover a good part of the range, whilst
    // dispensing with the need to clamp the final result.
    float d = 1.;

    // I've unrolled one of the loops. GPU architecture is a mystery to me, but I'm aware
    // they're not fond of nesting, branching, etc. My laptop GPU seems to hate everything,
    // including multiple loops. If it were a person, we wouldn't hang out.
    for(float j = -1.; j < 1.01; j++) {
        for(float i = -1.; i < 1.01; i++) {

            b = vec3(i, j, -1.);
            r = b - p + hash33(g+b);
            d = min(d, dot(r,r));

            b.z = 0.0;
            r = b - p + hash33(g+b);
            d = min(d, dot(r,r));

            b.z = 1.;
            r = b - p + hash33(g+b);
            d = min(d, dot(r,r));

        }
    }

    return d; // Range: [0, 1]
}

// Standard fBm function with some time dialation to give a parallax
// kind of effect. In other words, the position and time frequencies
// are changed at different rates from layer to layer.
//
float noiseLayers(in vec3 p) {

    // Normally, you'd just add a time vector to p, and be done with
    // it. However, in this instance, time is added seperately so that
    // its frequency can be changed at a different rate. p.z is thrown
    // in there just to distort things a little more.
    vec3 t = vec3(0., 0., p.z+(u_time*0.25)*2.0);

    const int iter = 5; // Just five layers is enough.
    float tot = 0., sum = 0., amp = 1.; // Total, sum, amplitude.

    for (int i = 0; i < iter; i++) {
        tot += voronoi(p + t) * amp; // Add the layer to the total.
        p *= 2.0; // Position multiplied by two.
        t *= 1.5; // Time multiplied by less than two.
        sum += amp; // Sum of amplitudes.
        amp *= 0.5; // Decrease successive layer amplitude, as normal.
    }

    return tot/sum; // Range: [0, 1].
}

void main()
{
    // Screen coordinates.
    vec2 uv = (gl_FragCoord.xy - u_resolution.xy*0.5) / u_resolution.y;

    // Shifting the central position around, just a little, to simulate a
    // moving camera, albeit a pretty lame one.
    uv += vec2(sin(u_time*0.5)*0.25, cos(u_time*0.5)*0.125);

    // Constructing the unit ray.
    vec3 rd = normalize(vec3(uv.x, uv.y, 3.1415926535898/7.));

    // Rotating the ray about the XY plane, to simulate a rolling camera.
    float cs = cos(u_time*0.25), si = sin(u_time*0.25);
    rd.xy *= mat2(cs, -si, si, cs);

    // Passing a unit ray multiple into the Voronoi layer function, which
    // is nothing more than an fBm setup with some time dialation.
    float c = noiseLayers(rd*3.);

    // Optional: Adding a bit of random noise for a subtle dust effect.
    // c = max(c + dot(hash33(rd)*2.-1., vec3(0.015)), 0.);

    // Coloring:

    // Nebula.
    // c *= sqrt(c)*1.5; // Contrast.
    c = pow(c, 1.333)*1.0;
    vec3 col = firePalette(c); // Palettization.
    col = mix(col, col.zyx*0.3+c*0.175, (1.+rd.x+rd.y)*0.45 ); // Color dispersion.

    // The fire palette on its own. Perhaps a little too much fire color.
    // c = pow(c, 1.33)*1.33;
    // vec3 col =  firePalette(c);

    // Black and white, just to keep the art students happy. :)
    // c *= sqrt(c)*1.5;
    // vec3 col = vec3(c);

    // Done.
    gl_FragColor = vec4(clamp(col, 0., 1.), 1.);
}
